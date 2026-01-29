// src/s03_smart_pointers/ex03_refcell.rs
use std::rc::Rc;
use std::cell::RefCell;

// 1. 定义交易池 (底层数据)
#[derive(Debug)]
struct Mempool {
    txs: Vec<String>,
}

// 2. 定义节点 (持有交易池的共享引用)
struct Node {
    id: u64,
    // 关键组合拳：Rc 让大家共享，RefCell 让大家修改
    pool: Rc<RefCell<Mempool>>, 
}

impl Node {
    fn new(id: u64, pool: Rc<RefCell<Mempool>>) -> Self {
        Node { id, pool }
    }

    // 提交交易
    fn submit_tx(&self, tx: &str) {
        // --- 关键动作慢放：borrow_mut() 的生命周期 ---
        
        // 第一步：申请锁 (Request)
        // 代码执行 self.pool.borrow_mut()。CPU 沿着栈上的指针找到堆上的 RcBox。
        // 运行时检查 (Runtime Check)：查看 borrow_flag（位于 RcBox 偏移 0x10 处）。
        // 如果 flag == 0：通过。
        // 如果 flag > 0：Panic! (报错：already borrowed: BorrowMutError)。
        //  因为 Rust 禁止同时存在可变和不可变借用。脏读写会导致不可预测的行为。
        // 如果 flag == -1：Panic! (报错：already borrowed: BorrowMutError)。
        //  因为 Rust 禁止同时存在两个可变借用。这是为了数据一致性。
        
        // 第二步：上锁 (Lock) & 第三步：发放凭证 (Guard Creation)
        // 检查通过，RefCell 将 borrow_flag 设为 -1。此时其他借用请求都会被拒绝，直到这个借用结束。
        // 返回一个 RefMut<'a, Mempool> 智能指针（即下面的 pool_guard）。
        let mut pool_guard = self.pool.borrow_mut();
        
        // 第四步：操作数据 (Mutation)
        // 通过实现了 DerefMut 的 pool_guard 向 Vec 推入数据。
        pool_guard.txs.push(format!("Node{}: {}", self.id, tx));
        println!("Node {} submitted tx.", self.id);
        
        // 第五步：自动解锁 (Drop & Unlock)
        // pool_guard 在这里离开作用域。利用 RAII 机制，RefMut 的 drop() 被调用。
        // 它负责将堆上的 borrow_flag 从 -1 改回 0。
    }

    fn print_pool(&self) {
        // borrow() 同样会触发运行时检查：将 borrow_flag 从 0 变为 1 (或 N+1)
        let pool_guard = self.pool.borrow();
        println!("Node {} sees pool: {:?}", self.id, pool_guard.txs);
        // 离开作用域，flag 减 1
    }
}

pub fn run() {
    println!("--- S03 Ex03: RefCell 内部可变性 ---");

    let shared_pool = Rc::new(RefCell::new(Mempool {
        txs: Vec::new(),
    }));

    let node1 = Node::new(1, Rc::clone(&shared_pool));
    let node2 = Node::new(2, Rc::clone(&shared_pool));

    node1.submit_tx("Mint 100 BTC");
    node2.print_pool();
    node2.submit_tx("Transfer 50 BTC");
    node1.print_pool();

    // ❌ 运行时 Panic 演示：
    // let borrow1 = shared_pool.borrow_mut(); 
    // let borrow2 = shared_pool.borrow_mut(); // Panic! 此时 flag 已经是 -1 了
}

// ==============================================================
// 内存全景图：俄罗斯套娃结构 (Rc<RefCell<Vec<String>>>)
// ==============================================================
// 
// 第一层：Stack (栈) —— 遥控器 (Node 结构体)
// +-----------+         +-------------------------------------------------------+
// | node1     |         | RcBox (分配在堆地址 0xHeapA)                            |
// |   id: 1   |         |-------------------------------------------------------|
// |   pool ---+-------->| 第二层：Heap (堆) —— 控制中心 (RcBox)                    |
// +-----------+         |-------------------------------------------------------|
//                       | Offset | Field        | Size | Value (Example)        |
// +-----------+         |--------|--------------|------|------------------------|
// | node2     |         | 0x00   | strong_count | 8 B  | 2 (node1, node2)       | <-- Rc 负责
// |   id: 2   |         | 0x08   | weak_count   | 8 B  | 0                      | <-- Rc 负责
// |   pool ---+-------->|--------|--------------|------|------------------------|
// +-----------+         | 0x10   | borrow_flag  | 8 B  | 0 (空闲)                | <-- RefCell 负责
//                       |        | (isize)      |      | 1..N (N个只读借用)      |
//                       |        |              |      | -1 (1个可变借用)        |
//                       |--------|--------------|------|------------------------|
//                       | 0x18   | value:       |      | 第三层：Heap (堆) 胖指针  |
//                       |        | Mempool.txs  | 24 B | (Vec 的元数据)          |
//                       |        |   - ptr -----+      | -> 指向 0xHeapB        |
//                       |        |   - cap      |      | 4                      |
//                       |        |   - len      |      | 2                      |
//                       +-------------------------------------------------------+
//                                                      |
//                                                      v
//                                       +---------------------------------------+
//                                       | 第四层：Heap (堆) —— 真正的数据区 (0xHeapB) |
//                                       |---------------------------------------|
//                                       | ["Tx1", "Tx2", ...]                   |
//                                       +---------------------------------------+

/*
深度对比解析：

1. 为什么需要 RefMut 这个中间人？
   如果 borrow_mut 直接返回裸引用 &mut Mempool，那么当引用使用完毕时，
   没有人去把堆内存偏移 0x10 处的 borrow_flag 改回 0。
   RefMut 的存在是为了利用 Rust 的 RAII 机制：只要 RefMut 还在，锁就在；
   RefMut 死了（Drop），锁就解开了。

2. 无缝融合与性能：
   Rc 并没有包含一个指针指向 RefCell。相反，RefCell 是直接嵌入在 Rc 管理的 RcBox 内存块里的。
   这意味着访问 Mempool 数据需要两次跳转（Double Indirection）：
   Node -> RcBox (0xHeapA) -> String Buffer (0xHeapB)。
   虽然比直接访问多了一层，但在绝大多数应用场景下，这个开销微乎其微。

总结：
- Rc：负责堆内存的存活（只要有人拿着钥匙，房间就不销毁）。
- RefCell：负责堆内存的借用规则（房间门上的计数器，运行时检查）。
- borrow_mut：修改计数器为 -1，并给你一个带自动恢复功能的句柄 (RefMut)。
*/












/* 
// src/s03_smart_pointers/ex03_refcell.rs
use std::rc::Rc;
use std::cell::RefCell;

// 1. 定义交易池 (底层数据)
#[derive(Debug)]
struct Mempool {
    txs: Vec<String>,
}

// 2. 定义节点 (持有交易池的共享引用)
struct Node {
    id: u64,
    // 关键组合拳：Rc 让大家共享，RefCell 让大家修改
    pool: Rc<RefCell<Mempool>>, 
}

impl Node {
    fn new(id: u64, pool: Rc<RefCell<Mempool>>) -> Self {
        Node { id, pool }
    }

    // 提交交易
    fn submit_tx(&self, tx: &str) {
        // 1. borrow_mut(): 运行时请求可变借用
        // 尝试获取写入权限，对应&mut self
        let mut pool_guard = self.pool.borrow_mut();
        
        pool_guard.txs.push(format!("Node{}: {}", self.id, tx));
        println!("Node {} submitted tx.", self.id);
        
        // pool_guard 在这里离开作用域，自动归还锁 (Drop)
    }

    // 读取交易
    fn print_pool(&self) {
        // 2. borrow(): 运行时请求不可变借用
        // 尝试获取读取权限，对应&self
        let pool_guard = self.pool.borrow();
        println!("Node {} sees pool: {:?}", self.id, pool_guard.txs);
    }
}

pub fn run() {
    println!("--- S03 Ex03: RefCell 内部可变性 ---");

    // 1. 创建共享的交易池
    let shared_pool = Rc::new(RefCell::new(Mempool {
        txs: Vec::new(),
    }));

    // 2. 创建两个节点，共享同一个池子
    let node1 = Node::new(1, Rc::clone(&shared_pool));
    let node2 = Node::new(2, Rc::clone(&shared_pool));

    // 3. Node 1 提交交易
    node1.submit_tx("Mint 100 BTC");

    // 4. Node 2 读取交易 (能看到 Node 1 的修改！)
    node2.print_pool();

    // 5. Node 2 再次提交
    node2.submit_tx("Transfer 50 BTC");
    node1.print_pool();

    // ❌ 费曼挑战：运行时 Panic
    // RefCell 把借用检查从编译期推迟到了运行期。
    // 如果你违反了规则（比如同时有两个可变借用），程序会直接崩溃。
    
    // 请尝试取消下面的注释，观察 Crash：
    // let borrow1 = shared_pool.borrow_mut(); // 拿走写锁
    // let borrow2 = shared_pool.borrow_mut(); // 再次尝试拿写锁 -> Panic!
}

// ==============================================================
// 解析：RefCell 的工作原理
// 栈 (Stack)                                 堆 (Heap)
// +-----------+                           +-------------------------------------------+
// | node1     |                           | RcBox (分配在堆上)                        |
// |   id: 1   |                           |-------------------------------------------|
// |   pool ---+-------------------------> | strong_count: 2 (node1, node2)          |
// +-----------+                           | weak_count:   0                           |
//                                         |-------------------------------------------|
// +-----------+                           | RefCell (包裹着数据)                      |
// | node2     |            +------------->| borrow_state: 1 (表示当前由1个借用者)     |
// |   id: 2   |            |              | value: Mempool                            |
// |   pool ---+------------+              |   + txs: Vec<String> ------------------> [Buffer "Tx1", "Tx2"]
// +-----------+                           +-------------------------------------------+

// 节点层：Node 结构体非常轻量，只包含一个 ID 和一个指向堆内存的指针（由 Rc 管理）。
// Rc 层：堆上首先是引用计数器，确保两个节点引用的是同一块内存。
// RefCell 层：在数据外面的一层薄壳，里面存储了一个动态借用标志（类似于一个整数计数器）。
//  当节点请求借用时，RefCell 会检查当前的借用状态，确保没有违反 Rust 的借用规则。
//  状态标记：
//    0：没有借用
//    正数 n：有 n 个不可变借用
//    负数 -1：有一个可变借用
// 数据层：最里面才是真正的 Mempool。
// ==============================================================


// [ Heap Allocation Block (RcBox) ]  <-- Rc::new 分配的这一整块
// |-------------------------------------------------------|
// | Offset | Field          | Size | Value (Example)      |
// |--------|----------------|------|----------------------|
// | 0x00   | strong_count   | 8 B  | 2 (Node1 + Node2)    | <-- Rc 负责
// | 0x08   | weak_count     | 8 B  | 0                    | <-- Rc 负责
// |--------|----------------|------|----------------------|
// | 0x10   | borrow_flag    | 8 B  | 0 (UNUSED)           | <-- RefCell 负责
// |        |                |      | 1 (READING)          |
// |        |                |      | -1 (WRITING)         |
// |--------|----------------|------|----------------------|
// | 0x18   | Mempool.txs    | 24 B | (Vec 的胖指针)        | <-- 数据本身
// |        |  - ptr         |      | -> 指向堆上另一处的字符数组
// |        |  - cap         |      | 4                    |
// |        |  - len         |      | 2                    |
// |-------------------------------------------------------|

// 1.无缝融合：Rc 并没有包含一个指针指向 RefCell。相反，RefCell 是直接嵌入在 Rc 管理的那块内存里的。
//   这就解释了为什么这种组合性能很高。

// 2.borrow_flag (借用状态字段)：这是 RefCell 唯一的开销。它就是一个简单的整数（isize）。
//   调用 borrow() 时：flag += 1。
//   调用 borrow_mut() 时：检查是否为 0。是则设为 -1，否则 Panic。

// 3.双重间接 (Double Indirection)：
//   Node.pool 指向 -> RcBox (0x00)
//   RcBox 里的 Mempool.txs.ptr 指向 -> String Buffer (交易数据)
//   结论：访问交易数据需要跳两次。这在极高性能场景（如高频交易）可能是瓶颈，但在应用层完全可接受。


*/

