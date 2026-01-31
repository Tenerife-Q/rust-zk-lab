// src/s04_concurrency/ex02_sync.rs
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/*
 业务逻辑 (Business Logic)
    这就好比 10 个柜员同时在给同一个银行账户存钱：

    1.初始化账户：创建一个银行账户，初始余额为 0。
    2.多线程操作：启动 10 个线程，模拟 10 个操作员。
    3.并发执行：每个线程都要做同一件事——给这个账户增加 10 块钱，并打印当前余额。
    4.安全控制：为了防止两个操作员同时修改账本导致金额算错（例如“竞争条件”），
        必须使用“锁”机制，确保同一时间只有一个线程在修改余额。
    5.汇总结果：等待所有人干完活，最后查看账户的总余额。预期结果应该是 100。
*/

pub fn run() {
    println!("--- S04 Ex02: 共享状态 (Arc + Mutex) ---");

    // 1. 定义共享数据：一个包含余额的账户
    // Arc 让数据可以被多线程拥有
    // Mutex 让数据可以被安全修改（互斥锁）
    let account = Arc::new(Mutex::new(0)); 
    
    let mut handles = vec![];

    // 2. 启动 10 个线程，每个线程存 10 块钱
    for i in 0..10 {
        // 克隆 Arc 指针：增加引用计数 (原子操作)
        let account_ref = Arc::clone(&account);

        let handle = thread::spawn(move || {
            // 3. 获取锁
            // lock() 会阻塞，直到拿到锁
            // unwrap() 是因为如果别的线程 panic 了，锁会"中毒"(Poisoned)，这里简化处理
            let mut num = account_ref.lock().unwrap();

            // 4. 修改数据
            *num += 10;
            println!("Thread {} deposited 10. Balance: {}", i, *num);
            
            // 锁在这里自动释放 (Drop)
        });
        handles.push(handle);
    }

    // 3. 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 4. 打印最终结果
    println!("Final Balance: {}", *account.lock().unwrap());
}

/*
 解析：
    Arc (Atomic Reference Counted)：
        允许多个线程共享同一份数据的所有权。
        内部使用原子操作来维护引用计数，确保线程安全。
    Mutex (Mutual Exclusion)：
        提供一种机制，确保同一时间只有一个线程可以访问被保护的数据。
        当一个线程锁定了 Mutex，其他线程必须等待，直到锁被释放。
    组合使用：
        Arc<Mutex<T>> 是 Rust 中常见的并发模式。
        Arc 负责多线程间的所有权共享，Mutex 负责数据的安全修改。
    lock() 方法：
        当调用 lock() 时，如果没有其他线程持有锁，它会立即返回一个 MutexGuard。
        如果锁已经被其他线程持有，调用线程会阻塞，直到锁可用。
    自动释放锁：
        当 MutexGuard 超出作用域时，Rust 会自动调用 drop() 方法释放锁，
        确保不会发生死锁（Deadlock）。
*/  




/* 
内存结构画图分析

let account = Arc::new(Mutex::new(0));

[ 栈 Stack (Main Thread) ]           [ 堆 Heap (ArcInner 内存块) ]
+----------------------+             +---------------------------------------------+
| account (变量)       |             | ArcInner<Mutex<i32>>                        |
| [ ptr ] -------------------------> | +-----------------------------------------+ |
| [ phantom ]          |             | | strong_count: AtomicUsize (值: 1)       | |
+----------------------+             | | weak_count:   AtomicUsize (值: 1)       | |
                                     | | data: Mutex<i32>                        | |
                                     | | +-------------------------------------+ | |
                                     | | | inner: sys::MovableMutex (系统锁)   | | | <--- 此时是 Unlocked
                                     | | | poison: poison::Flag (未中毒)       | | |
                                     | | | data: UnsafeCell<i32> (值: 0)       | | | <--- 真正的数据在这里
                                     | | +-------------------------------------+ | |
                                     | +-----------------------------------------+ |
                                     +---------------------------------------------+
底层剖析
1. Arc 的结构 (std::sync::Arc):
    在栈上，account 本质上只是一个裸指针 (NonNull<ArcInner<T>>)。
    它没有任何数据字段，只是指向堆。
    Arc 并没有“存”数据，它只是管理元数据（引用计数）。

2. ArcInner (堆上隐藏的结构):
    这是 Rust 自动分配的一块连续内存。
    strong_count: 强引用计数。当前值为 1，代表只有 account 拥有它。
    weak_count: 弱引用计数。初始通常也是 1（为了处理 Arc 自身的生命周期，细节较复杂），不影响数据释放逻辑。
    
3. Mutex 的结构 (std::sync::Mutex): 
    Mutex 被包裹在 ArcInner 内部。
    inner (sys::Mutex): 这是一个跟操作系统绑定的字段。
        在 Linux 上，它内部通常基于 futex (Fast Userspace Mutex) 实现。
        它是一个只有 4 字节的整数，0 代表无锁，1 代表有锁，2 代表有锁且有线程在等待（Contention）。
        这部分非常关键，它是实现阻塞的物理基础。
    data (UnsafeCell<i32>):
        Rust 的正常借用规则禁止在没有 mut 的情况下修改数据。
        UnsafeCell 是唯一能绕过这个规则的后门。Mutex 内部使用它来存放真正的 0。
        只有通过 Mutex 的逻辑检查后，它才会给你一个指向这个 UnsafeCell 内部的 &mut i32。





let account_ref = Arc::clone(&account);
thread::spawn(move || { ... });

[ 栈 Stack (Main Thread) ]    [ 堆 Heap (不变，但计数增加) ]        [ 栈 Stack (Thread-1) ]
+----------------------+      +-----------------------------+       +-------------------------+
| account              |      | ArcInner                    |       | account_ref (moved here)|
| [ ptr ] ------------------> | strong_count: 2 (Atomic)    | <------ [ ptr ]                 |
+----------------------+      | ...                         |       +-------------------------+
                              | data: Mutex<i32>            |
                              +-----------------------------+

底层剖析
1. Arc 克隆 (Arc::clone):
    Arc::clone 并不会复制堆上的数据。Mutex 和 i32 都保持不变。
    它只是增加了 strong_count 的值（从 1 变成 2），表示现在有两个 Arc 指向同一块数据。
    这个操作是原子性的，确保在多线程环境下不会出错。
    这样不会像普通加法一样出错（例如两个线程同时读到 1，然后都写回 2）。
    最后导致内存泄漏或提前释放。
2. 所有权转移 (move 关键字):
    当我们把闭包传给 thread::spawn 时，必须使用 move 关键字。
    这会把 account_ref 的所有权从主线程转移到新线程。
    这样，主线程和子线程都能安全地使用同一份数据，而不会发生悬垂引用。
    此时，主线程持有 account，子线程持有 account_ref。它们指向同一个 ArcInner。
3. 多线程访问:
    现在，主线程和子线程都有一个 Arc 指向同一块 Mutex<i32> 数据。
    每个线程都可以调用 lock() 来获取锁，修改数据，然后释放锁。
    Arc 和 Mutex 会确保这个过程是线程安全的，不会发生数据竞争。



let mut num = account_ref.lock().unwrap();
*num += 10;

[ 栈 Stack (Thread-1) ]                               [ 堆 Heap (ArcInner) ]
+---------------------------------------+             +-----------------------------------------+
| account_ref (Arc)                     | ----------> | strong_count: 2                         |
+---------------------------------------+             | ...                                     |
| num (MutexGuard)                      |             | data: Mutex<i32>                        |
| +-----------------------------------+ |    指向锁   | +-------------------------------------+ |
| | lock: &Mutex<i32>  [ ptr ] ---------------------> | | inner: sys::MovableMutex (LOCKED)   | | <--- 状态变了！
| | poison: poison::Guard             | |             | | ...                                 | |
| +-----------------------------------+ |             | | data: UnsafeCell<i32> (值: 10)      | | <--- 正在修改
                                                      | +-------------------------------------+ |
                                                      +-----------------------------------------+

底层剖析
1. 获取锁 (lock 方法):
    当调用 account_ref.lock() 时，Mutex 会检查 inner 字段的状态。
    - 如果是 0（Unlocked），它会把它改成 1（Locked），表示当前线程持有锁。
    - 如果是 1 或 2（Locked），当前线程会被阻塞，直到锁可用。
    这个过程是通过操作系统的原子操作和调度机制实现的，确保线程安全。
2. MutexGuard(num):
    lock() 返回一个 MutexGuard，它是一个智能指针，持有对 Mutex 的引用。
    当 MutexGuard 超出作用域时，它会自动调用 drop() 方法，释放锁。
    这确保了即使线程发生 panic，锁也会被正确释放，避免死锁。
3. 修改数据(*num += 10):
    通过解引用 MutexGuard（*num），我们可以安全地访问和修改内部的 i32 数据。
    这里我们把余额增加了 10。
4. 自动释放锁(move || { ... }结束):
    当 MutexGuard 离开作用域时，Rust 会自动调用它的 drop() 方法。
    这会把 inner 字段的状态从 1（Locked）改回 0（Unlocked），允许其他线程获取锁。
    具体来说
        Drop num (MutexGuard):
            作用域结束，num 被销毁。MutexGuard::drop 被调用。
            它会去堆上的 Mutex 里的 inner 字段，将其状态通过原子操作改回 0 (Unlocked)。
            唤醒 (Wake up): 它会检查系统队列里有没有别的线程在等这把锁。
            如果有，调用系统调用（如 futex_wake）把那个倒霉蛋叫醒。

        Drop account_ref (Arc):
            account_ref 离开作用域。Arc::drop 被调用。
            原子减法: LOCK XADD (减1)。strong_count 从 2 变回 1。
            检查计数：如果此时计数变为了 0，它负责释放堆内存 (free)。
            在这里，因为主线程的 account 还在，计数只是变回 1，内存得以保留。
*/