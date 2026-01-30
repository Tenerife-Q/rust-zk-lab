// src/s04_concurrency/ex01_thread.rs
use std::thread;
use std::time::Duration;

pub fn run() {
    println!("--- S04 Ex01: 线程基础 ---");

    // 1. 定义一个难以计算的任务 (模拟挖矿难度)
    let difficulty = 3; 
    let block_data = String::from("Block#100: [Tx1, Tx2]");

    println!("Main: 开始分发挖矿任务...");

    // 2. 启动一个新线程
    // thread::spawn 接收一个闭包 FnOnce
    // ❌ 陷阱 1：闭包试图借用主线程的变量
    // 编译器会报错：closure may outlive the current function
    // 提示：子线程有自己的栈，它不能依赖主线程栈上的引用（因为主线程可能先死）
    // 请修复：添加 move 关键字
    let handle = thread::spawn(move || {
        println!("  [Miner] 开始计算哈希，难度: {}...", difficulty);
        println!("  [Miner] 区块数据: {}", block_data); // 这里借用了 block_data
        
        // 模拟耗时计算
        thread::sleep(Duration::from_secs(2)); 
        
        println!("  [Miner] ⛏️ 挖矿成功！Hash: 000abc...");
    });

    // 3. 主线程继续做其他事
    println!("Main: 我在做网络心跳检测...");
    thread::sleep(Duration::from_millis(500));

    // ❌ 陷阱 2：主线程提前退出
    // 如果不等待子线程，主线程运行完这里就会退出程序 (Process Exit)。
    // 整个进程被杀死，子线程还没挖完就被强制终止了。
    // 提示：使用 handle.join() 来阻塞等待子线程结束
    
    // 请在此处添加代码修复陷阱 2
    handle.join().expect("子线程出错");
    
    println!("Main: 任务全部完成，安全退出。");
}