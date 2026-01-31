// src/s04_concurrency/ex03_channel.rs
use std::sync::mpsc; // mpsc = Multiple Producer, Single Consumer
use std::thread;
use std::time::Duration;

/*
一、 核心思想：并发哲学的转变

在 S04 Ex02 (Arc+Mutex) 中，并发是通过**“锁”**来管理的，大家抢着去修改同一块内存。
而这份代码展示了 Rust 的另一条并发准则：

> "Do not communicate by sharing memory; instead, share memory by communicating."
> 不要通过共享内存来通信，而要通过通信来共享内存。

业务场景：模拟区块链交易池
- 生产者 (Wallets)：多个线程 (tx, tx1) 异步产生交易数据。
- 消费者 (Node)：单个主线程 (rx) 接收并打包数据。
- 优势：数据像水流一样，由上游流向下游，解耦了生产与消费，且数据的所有权随消息物理转移，根除了数据竞争。
*/

pub fn run() {
    println!("--- S04 Ex03: 消息传递 (Channel) ---");

    // 1. 创建频道
    // tx = Transmitter (发送端), rx = Receiver (接收端)
    // mpsc::channel() 返回一个元组 (tx, rx)。泛型类型由后续send的数据推断
    let (tx, rx) = mpsc::channel();

    // 2. 启动生产者线程 (模拟钱包)
    // 我们可以克隆 tx，让多个钱包同时发送
    let tx1 = tx.clone();
    thread::spawn(move || {
        let txs = vec!["Tx_A1", "Tx_A2", "Tx_A3"];
        for t in txs {
            println!("Wallet A sending: {}", t);
            // send() 会转移数据的所有权
            tx1.send(String::from(t)).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
    });

    // 3. 启动第二个生产者
    thread::spawn(move || {
        let txs = vec!["Tx_B1", "Tx_B2"];
        for t in txs {
            println!("Wallet B sending: {}", t);
            tx.send(String::from(t)).unwrap(); // 使用原始的 tx
            thread::sleep(Duration::from_millis(300));
        }
    });

    // 4. 主线程作为消费者 (模拟打包节点)
    println!("Node: Waiting for transactions...");
    
    // rx 实现了 Iterator，所以可以直接用 for 循环接收
    // 这个循环会阻塞，直到所有 tx 都被 Drop (即发送端全部关闭)
    for received in rx {
        println!("Node: Got {}", received);
    }

    println!("Node: All senders disconnected. Exiting.");
}


/* 
二、 内部机制深度解剖：从代码行到 CPU 缓存一致性 (Expert Level)

   [ 线程 A (Main) ]                 [ 堆内存 (Heap / Channel Packet) ]                [ 线程 B (Worker) ]
  +-----------------+  (Ownership)  +---------------------------------------------+   +-----------------+
  | rx (Recv Handle)| ------------> |  Shared State (共享状态块)                  |   | tx1 (Send Handle)|
  | inner: *mut Pkt |               |                                             |   | inner: *mut Pkt |
  +-----------------+               |      [ Cache Line 1 (64 bytes) ]            |   +-----------------+
                                    |  [ head (AtomicPtr) ]: 消费者只读写这里     |           |
                                    |  [ ... Padding (填充数据) ...         ]     | <---------+
                                    |    (防止 False Sharing 导致缓存抖动)        |
                                    |                                             |
                                    |      [ Cache Line 2 (64 bytes) ]            |
                                    |  [ tail (AtomicPtr) ]: 生产者争抢这里       | <---------+
                                    |  [ status (AtomicUsize) ]: 状态机           |           |
                                    +----------------------|----------------------+           |
                                                           |                                  |
                                            +--------------v---------------------------+      |
                                            |       Data Node (堆节点)                 |      |
                                            |  [ data: T (String: ptr/len/cap) ]      |      |
                                            |  [ next: AtomicPtr (下一跳)      ] <-----+------+
                                            +------------------------------------------+

1. 初始化阶段：Arc-like 引用计数
--------------------------------------------------------------------------------------
   let (tx, rx) = mpsc::channel();
   let tx1 = tx.clone();
--------------------------------------------------------------------------------------
   - 分配 (Allocation): `mpsc::channel()` 是一个异步无界队列。Rust 在堆上请求一块连续内存初始化 `Packet`。
   - 引用计数 (Ref Counting): 这里并没有像 Arc 那样有显式的 Strong/Weak 计数，而是：
     Sender 数量由 `channels` 计数器维护。
     Receiver 独占 `port` 端。
   - Clone: `tx.clone()` 仅仅是原子递增 Packet 中的发送者计数。
     注意：这比复制整个队列快得多，但也意味着所有 Sender 都在争抢同一个 `tail` 指针。


2. 发送数据：原子指令与内存序 (Atomicity & Memory Ordering)
--------------------------------------------------------------------------------------
   tx1.send(msg).unwrap();
--------------------------------------------------------------------------------------
   A. 准备数据 (Preparation)
      `msg` (String) 先在线程栈上准备好。
      并在堆上分配一个新的 `Node`，将 `msg` 移动 (memcpy) 进去。
   
   B. 无锁入队 (Lock-Free Enqueue)
      关键指令：`AtomicPtr::swap(new_node, Ordering::Release)`
      1. CPU 硬件层面的 `LOCK XCHG` 指令，瞬间将 `tail` 指向 `new_node`，并返回 `old_tail`。
      2. Memory Ordering (Release): 这一步至关重要。它保证了在此指令之前的所有内存写入（即 Node 里的数据）
         对随后执行 Acquire 操作的线程（消费者）是**绝对可见**的。
      3. 链接: `old_tail.next.store(new_node, Ordering::Relaxed)`。将断开的链表补上。
      
   C. 背压警告 (Backpressure Warning)
      注意 `send` 永远不会阻塞。如果消费者处理过慢，`Nodes` 会在堆上无限堆积，
      最终导致 OOM (Out of Memory)。生产环境常推荐 `sync_channel` (有界队列)。


3. 接收数据：缓存优化与性能 (Cache Efficiency)
--------------------------------------------------------------------------------------
   for received in rx { ... }
--------------------------------------------------------------------------------------
   A. 缓存行填充 (Cache Padding) - 极致细节
      注意架构图中 `head` 和 `tail` 被放在了不同的 Cache Line 中。
      - 如果它们挨在一起，当 Main 线程修改 `head` 时，CPU 会强制 Worker 线程的 L1 Cache 失效（视为脏数据）。
      - Worker 线程下次访问 `tail` 必须去 L3 或主存重新拉取，这叫“伪共享 (False Sharing)”。
      - Rust 标准库通过插入无意义的字节填充，杜绝了这种性能杀手。

   B. 快速路径 (Fast Path)
      Main 线程独占 `head`。读取 `head` 不需要昂贵的原子指令 (或者仅需 Relaxed)。
      如果 `head != tail`，说明有数据，直接取走，零锁开销。

   C. 慢速路径 (Slow Path)
      如果 `head == tail`，队列空。
      Main 线程修改 `status` 为 BLOCKED，调用 OS 原语 (`futex_wait` on Linux) 挂起。
      此时 OS 调度器将其移出运行队列，不再消耗 CPU 周期。


4. 异常处理与生命周期 (Robustness)
--------------------------------------------------------------------------------------
   - Sender Drop: 引用计数减 1。归零时，Channel 标记为 Disconnected。
   - Receiver Drop: 如果接收端先挂了，`tx.send()` 会立即返回 `Err(SendError)`。
     Rust 的这一机制防止了生产者向“黑洞”发送数据而不知情。
*/