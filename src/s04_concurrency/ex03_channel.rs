// src/s04_concurrency/ex03_channel.rs
use std::sync::mpsc; // mpsc = Multiple Producer, Single Consumer
use std::thread;
use std::time::Duration;

/*
一、 核心思想：并发哲学的转变
在 Arc<Mutex<T>> 中，并发是通过**“锁”**来管理的，大家抢着去修改同一块内存。
而这份代码展示了 Rust 的另一条并发准则：

"Do not communicate by sharing memory; instead, share memory by communicating."
不要通过共享内存来通信，而要通过通信来共享内存。

业务场景：模拟区块链交易池。
生产者 (Wallets)：多个线程 (tx, tx1) 异步产生交易数据。
消费者 (Node)：单个主线程 (rx) 接收并打包数据。
单向流动：数据像水流一样，从上游流向下游，逻辑线性且清晰。
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


/* 二、 内部机制解析：Channel 的数据结构与工作原理


   [ 线程 A (Main) ]                [ 堆内存 (Heap / Shared State) ]               [ 线程 B (Worker) ]
  +-----------------+            +------------------------------------+           +-----------------+
  | rx (Receiver)   |            |  Internal Queue Packet (共享包)    |           | tx1 (Sender)    |
  | - inner_ptr  -----------------> [ status: Connected             ] <-------------- inner_ptr -   |
  +-----------------+            |  [ ref_count (senders): 2      ]   |           +-----------------+
                                 |  [ port_queue (receivers): 1   ]   |
                                 |                                    |
                                 |  [ 队列头指针 (Head) ] ------------+
                                 |           ^                        |
                                 |  [ 队列尾指针 (Tail) ] --+         |
                                 |  [ 信号量 (Signal) ]     |         |
                                 +--------------------------|---------+
                                                            |
                                 +--------------------------|---------+
                                 |       Data Node (数据节点)         |
                                 |  [ value: "Tx_A1" (String) ] <-----+
                                 |  [ next:  null             ]       |
                                 +------------------------------------+
                        
关键字段解析
inner_ptr: tx 和 rx 只是栈上的轻量级句柄（Handle），
    它们内部只有一个非空指针 (NonNull) 指向堆上的共享状态。
Data Node: 这是一个单向链表。
    Rust 的 mpsc 实现（特别是针对非同步通道）
    通常使用了 Lock-free Queue (无锁队列) 的变体（如 Michael-Scott 队列算法），
    确保像 send 这样的操作极快，几乎不需要操作系统级别的锁。
Signal/Park: 这是最重要的字段。
    当队列为空时，消费者会通过系统调用（如 Linux 的 futex）在这个地址上挂起 (Park) 线程，
    不再通过 CPU 空转，直到被唤醒。




let(tx, rx) = mpsc::channel();
let tx1 = tx.clone();

channel(): 
    调用 malloc 在堆上分配上述的 Queue Packet。初始化引用计数 senders=1, receivers=1。
tx.clone():
    原子操作: 对堆上的 ref_count 执行原子加一 (AtomicUsize::fetch_add)。
    结果: 现在的 senders=2。注意：没有创建新的队列，只是增加了一个指向它的“遥控器”。






// 在线程中
tx1.send(String::from("Tx_A1")).unwrap();

send(value):
    1. 创建 Data Node:
        在堆上分配一个新的 Data Node，存放 value 和 next=null。
    2. 入队操作 (Enqueue):
        使用原子操作将新节点链接到队列尾部 (Tail)。
        这个过程是无锁的，允许多个发送者同时调用 send 而不会阻塞。
    3. 信号通知 (Signal):
        如果有消费者在等待（Parked），通过信号量唤醒它们，让它们知道有新数据到达。
栈上分配: 
    线程 B 在自己的栈上创建了一个 String 结构（ptr, len, cap）。
物理移动 (Memcpy):
    send 接受值类型 T。编译器将这个 String 的结构体字节流复制进队列的一个新节点中。
逻辑失效 (Invalidation):
    发送端的 String 变量在 send 后变得不可用（所有权被转移）。
    Rust 编译器会阻止你再使用这个变量，确保内存安全。
唤醒 (Wake up):
    tx1 入队成功后，会原子检查接收端状态。如果发现 rx 正在休眠 (Blocked)，它会发出信号唤醒主线程。









// 主线程
for received in rx { ... }
检查队列: 有数据吗？
    有: 取出数据，所有权转移给 received 变量，处理业务。
    无: 主线程告诉操作系统：“我没活干了，把我挂起 (Park)”。线程进入 Sleep 状态，CPU 占用率降为 0%。

优雅退出 (Graceful Shutdown):
    这个循环什么时候结束？这是 Channel 的魔法。
    当所有的 tx (tx 和 tx1) 都离开作用域被 Drop 时，堆上的 ref_count 归零。
    队列状态更新为 Disconnected。
    rx.recv() 检测到“队列为空 且 连接已断开”，返回 Err (迭代器返回 None)，循环终止。

*/