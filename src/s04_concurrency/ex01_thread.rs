// src/s04_concurrency/ex01_thread.rs
use std::thread;
use std::time::Duration;

pub fn run() {
    println!("--- S04 Ex01: 线程基础 ---");

    // 1. 定义一个难以计算的任务 (模拟挖矿难度)
    let difficulty = 3; 
    let block_data = String::from("Block#100: [Tx1, Tx2]");

    println!("Main: 开始分发挖矿任务...");

    // 2. 启动一个新线程 这里是新建立了一个子线程
    // thread::spawn 接收一个闭包 FnOnce
    // ❌ 陷阱 1：闭包试图借用主线程的变量
    // 编译器会报错：closure may outlive the current function
    // 提示：子线程有自己的栈，它不能依赖主线程栈上的引用（因为主线程可能先死）
    // 请修复：添加 move 关键字
    let handle = thread::spawn(move || {
        println!("  [Miner] 开始计算哈希，难度: {}...", difficulty);
        println!("  [Miner] 区块数据: {}", block_data); // 这里借用了 block_data
        /*
        thread::spawn：
            接受一个闭包 || { ... } 作为线程的主函数。
            返回值 JoinHandle：一个句柄，用于控制和等待该线程。
        move 关键字（关键！）：
            如果不加 move：闭包默认会尝试借用（Reference）外面的变量 &difficulty 和 &block_data。
        内存安全隐患：Rust 编译器不知道这个新线程会跑多久。如果主线程（run 函数）先跑完了，
            主线程的栈帧（Stack Frame）会被销毁，difficulty 和 block_data 随之消失。
            此时，子线程如果还持有对它们的引用，就会导致悬垂指针（Dangling Pointer）。
        move 的作用：强制将闭包捕获的变量的所有权（Ownership）转移（Move）或者复制（Copy）到子线程的栈空间中。
            difficulty 是 Copy 类型，被复制了一份到子线程栈。
            block_data 是 String（非 Copy），即所有权转移，原主线程变量失效。
            主线程栈上的 block_data 胖指针失效，子线程栈上拥有了一个新的胖指针，
            指向同一个堆内存区域（只要不发生写时复制/重分配）。
        底层原理：这是 Rust 实现“无数据竞争（Data Race Freedom）”的关键手段之一。
            通过所有权系统，保证同一时间只有一个线程能随意修改该数据（或者像这里一样完全转移走）。
         */
        
        // 模拟耗时计算
        thread::sleep(Duration::from_secs(2)); 
        
        println!("  [Miner] ⛏️ 挖矿成功！Hash: 000abc...");
    });

    // 3. 主线程继续做其他事 这里是main主线程 
    // 并行和并发的区别：并行是同时运行多个任务（多核CPU），并发是任务交替进行（单核CPU）
    // 所以这是并发 那你上面还说是并行？
    // 答：这里的并发是指主线程和子线程交替运行
    println!("Main: 我在做网络心跳检测...");
    thread::sleep(Duration::from_millis(500));
    /*
    并发（Concurrency）：这两个 sleep 是同时（或交替）发生的。
        操作系统调度器会在两个线程之间快速切换（Context Switch）。
    输出顺序：通常你会先看到主线程的“心跳检测”（因为它只睡500ms），或者根据调度器的策略交错出现。

    预期输出：
        Main: 我在做网络心跳检测...
        [Miner] 开始计算哈希，难度: 3...
        [Miner] 区块数据: Block#100: [Tx1, Tx2]
        [Miner] ⛏️ 挖矿成功！Hash: 000abc...
        解释：主线程和子线程并发执行，主线程的输出可能会先出现，因为它睡眠时间更短。
    实际输出：
        由于线程调度的不确定性，实际输出顺序可能会有所不同。
        Main: 开始分发挖矿任务...
        Main: 我在做网络心跳检测...
            [Miner] 开始计算哈希，难度: 3...
            [Miner] 区块数据: Block#100: [Tx1, Tx2]
        0
        0
            [Miner] ⛏️ 挖矿成功！Hash: 000abc...
        Main: 任务全部完成，安全退出。
     */

    // ❌ 陷阱 2：主线程提前退出
    // 如果不等待子线程，主线程运行完这里就会退出程序 (Process Exit).
    // 整个进程被杀死，子线程还没挖完就被强制终止了。
    // 提示：使用 handle.join() 来阻塞等待子线程结束
    
    // 请在此处添加代码修复陷阱 2
    handle.join().expect("子线程出错");
    /*
    handle.join()：
        作用：阻塞当前线程（主线程），直到 handle 对应的子线程执行完毕退出。

    为什么必须 Join？：如果不 Join，主函数 run() 结束，main 函数结束，整个进程（Process）会立即退出。
        操作系统会回收进程的所有资源，正在运行的子线程会被直接终止（Terminate），导致任务未完成。

    返回值 Result：
        如果子线程正常结束，返回 Ok(T)，其中 T 是闭包的返回值（本例中是 ()）。
        如果子线程发生了 Panic（崩溃），返回 Err(...)。
        这也是 Rust 线程隔离的一个特性：一个线程 Panic 不会直接导致整个程序崩溃（除非是主线程），父线程可以捕获这个 Panic。
    
    注意事项：
        调用 join() 会阻塞当前线程，直到子线程结束。如果子线程执行时间较长，主线程会等待较久。
        如果子线程在执行过程中发生 Panic，调用 join() 会返回一个 Err，父线程可以选择如何处理这个错误。
     */
    
    println!("Main: 任务全部完成，安全退出。");
}


// [ 堆内存 (Heap) ]
//       ^
//       | (指向 "Block#100...")
//       |
// +----------------------+        转移发生 (move)       +-------------------------+
// |    主线程栈 (Main)    |  =======================>  |     子线程栈 (Miner)     |
// +----------------------+                            +-------------------------+
// | difficulty: 3        |  --------(Copy)--------->  | difficulty: 3           |
// | block_data (ptr,...) |  --------(Move)--------->  | block_data (ptr,...)    |
// | handle (JoinHandle)  |                            | (拥有对堆内存的所有权)      |
// +----------------------+                            +-------------------------+
//        |
//        | 等待 (join)
//        v