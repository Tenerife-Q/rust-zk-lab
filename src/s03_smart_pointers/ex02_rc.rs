// src/s03_smart_pointers/ex02_rc.rs
use std::rc::Rc;

#[derive(Debug)]
struct Block {
    id: u64,
    // data: String, // 简化省略
    
    // ❌ 如果用 Box，只能有一个父亲
    // parent: Option<Box<Block>>,
    
    // ✅ 使用 Rc，允许多个"儿子"共享同一个"父亲"
    // Rc = Reference Counted Smart Pointer
    parent: Option<Rc<Block>>, 
}

impl Block {
    fn new(id: u64, parent: Option<Rc<Block>>) -> Self {
        Block { id, parent }
    }
}

pub fn run() {
    println!("--- S03 Ex02: Rc 共享所有权 (DAG) ---");

    // 1. 创建创世块 (Genesis)
    // 把它装进 Rc 飞船，准备被共享
    // Rc::new() 会在堆上分配内存，并返回一个 Rc 指针，初始化引用计数为 1
    let genesis = Rc::new(Block::new(0, None));
    
    println!("Genesis initial refs: {}", Rc::strong_count(&genesis));

    // 2. 创建区块 1，指向 Genesis
    // Rc::clone(&genesis) 并不是拷贝数据，而是增加引用计数
    // 把 Rc 指针（genesis）的引用计数加 1，然后返回一个新的 Rc 指针，指向同一个堆地址
    // 为什么要返回新的 Rc 指针？因为每个 Rc 变量都需要自己的指针实例
    let block1 = Block::new(1, Some(Rc::clone(&genesis)));
    println!("Genesis refs after block1: {}", Rc::strong_count(&genesis));

    // 3. 创建区块 2，也指向 Genesis (形成了 DAG 结构)
    let block2 = Block::new(2, Some(Rc::clone(&genesis)));
    println!("Genesis refs after block2: {}", Rc::strong_count(&genesis));
    // strong_count(): 获取当前 Rc 指针的强引用计数（有多少个 Rc 指针指向同一个堆地址）

    // 4. 销毁区块 1
    drop(block1);
    println!("Genesis refs after block1 dropped: {}", Rc::strong_count(&genesis));
    // strong_count 变成 2，因为 block2和Genesis 还在引用它

    // ❌ 费曼挑战：Rc 的不可变性
    // Rc 允许共享，但代价是什么？
    // Rc<T> 只能提供对 T 的不可变引用 (immutable reference)。
    // 这意味着你不能通过 Rc 来修改它所指向的数据。
    // 试着修改 genesis 的 id：
    // 请尝试取消下面这行的注释：
    //genesis.id = 100; 
    
// 观察报错。为什么有了 Rc 就不能随意修改数据了？
//     因为“共享”意味着“不可变”。
// 如果 Rust 允许你通过 genesis 修改 ID，那么 block1 和 block2 看到的 ID 也会突然在它们毫不知情的情况下改变。
// 这会引发严重的数据竞争（Data Race），尤其是在并发场景下（虽然 Rc 是单线程的，但 Rust 的借用规则是通用的）。
// 口诀：共享不可变，可变不共享。

// 拓展：如果我既要共享又要修改怎么办？
// 你需要 内部可变性 (Interior Mutability)。
// 组合拳：Rc<RefCell<T>>。
// Rc 负责让多个人拥有它。
// RefCell 负责在不可变引用的内部提供可变修改的能力（运行时检查借用规则）。
// 这是下一节课最常见的模式。

    // 5. 销毁区块 2
    drop(block2);
    println!("Genesis refs after block2 dropped: {}", Rc::strong_count(&genesis));

    // 6. 当最后一个 Rc 被丢弃时，内存会被自动释放
}
//==========================================
//        Stack (栈)                                    Heap (堆)
// +-----------------------+                    +--------------------------+
// | genesis (Rc ptr)      | -----------------> | RcBox (控制块)           |
// +-----------------------+                    |--------------------------|
//                                              | strong_count: 3          | 所有者: genesis,
// +-----------------------+                    | weak_count:   0          |        block1.parent,
// | block1 (struct)       |           -------> |--------------------------|        block2.parent
// | - id: 1               |           |        | Block (数据)             |
// | - parent: Rc ptr      | -----------        | - id: 0                  |
// +-----------------------+           |        | - parent: None           |
//                                     |        +--------------------------+
// +-----------------------+           |
// | block2 (struct)       |           |
// | - id: 2               |           |
// | - parent: Rc ptr      | -----------
// +-----------------------+

// RcBox：Rc 在堆上不仅仅存数据，还额外存了两个计数器（strong 和 weak）。
// 共享：genesis 变量、block1 中的 parent 字段、block2 中的 parent 字段，这三个地方的指针完全一样，都指向同一个堆地址。
// 生命周期管理：
//      当 block1 销毁时，drop 会让 count - 1。
//      当 block2 销毁时，drop 会让 count - 1。
//      当 genesis 变量离开作用域时，drop 会让 count - 1。
// 归零：当 count 变成 0 时，Rust 才会真正释放堆上的这个 Block 内存。
//==========================================