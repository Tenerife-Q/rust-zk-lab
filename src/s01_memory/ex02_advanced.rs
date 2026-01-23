// src/s01_memory.rs

#[derive(Debug)]
struct Transaction {
    id: u64,
    payload: String, // 交易数据，堆内存
}

#[derive(Debug)]
struct Mempool {
    txs: Vec<Transaction>, // 交易列表
}

impl Mempool {
    fn new() -> Mempool {
        Mempool { txs: Vec::new() }
    }

    fn add(&mut self, tx: Transaction) {
        self.txs.push(tx); // 所有权移入 Vec
    }

    // ❌ 陷阱 1: 集合中的所有权移动
    // 场景：矿工想从交易池里“拿走”第一笔交易去打包
    // 提示：Vec 拥有交易的所有权，直接用索引 [0] 能拿走吗？
    fn pop_first(&mut self) -> Option<Transaction> {
        if self.txs.is_empty() {
            return None;
        }
        
        // 错误写法：
        // let tx = self.txs[0]; // 编译器会报错：cannot move out of index
        // return Some(tx);

        // 请修复上面，使用 Vec 提供的正确方法弹出元素

        let tx = self.txs.remove(0); // 正确写法：使用 remove 方法移除并返回第一个元素
        Some(tx)
        
        // None // 临时占位，修复后删掉这行

        /*
        出错原因：
            Vec 拥有其元素的所有权。尝试使用 self.txs[0] 直接获取元素会发生所有权转移，
            这会留下“空洞”，Rust 禁止这种操作。

        修复方案：
            使用 Vec::remove(index) 方法。它会将元素移出并把后面的元素向前移动填补空缺，
            从而安全地交出所有权

        底层原理：
            remove 内部使用了 ptr::read。它直接从内存地址拷贝字节到新位置，
            并利用 ptr::copy 把后面的元素往前挪。
            Rust 编译器知道这一块内存现在是“空”的，所以不会对原本 index 0 的位置重复调用 drop。
        
        本质上是：
            1. 读取 index 0 的数据到新位置（所有权转移）
            2. 将后续元素往前移动一位
            3. 更新 Vec 的长度，避免重复 drop 所造成的 double free
         */
    }

    // ❌ 陷阱 2: 悬垂引用 (Dangling Reference)
    // 场景：我们想获取最新交易的 payload 的切片用于打印日志
    // 这是一个非常隐蔽的错误，请仔细阅读报错
    fn get_latest_payload_preview(&self) -> &str {
        if let Some(tx) = self.txs.last() {
            // 假设我们要对 payload 做一些处理（比如截取前4位）再返回
            // let temp_str = tx.payload.clone(); // 克隆了一份数据到局部变量
            // return &temp_str[0..4];            // 返回局部变量的引用
            return &tx.payload[0..4]; // fix: 直接返回堆上数据的切片，避免悬垂引用
        }
        "Empty"

        /*
        出错原因：
            原代码中 let temp_str = tx.payload.clone(); 在 if 块内部创建了一个新的 String（局部变量）。
            当函数返回时，temp_str 被销毁，
            因此返回指向它的引用 &temp_str[0..4] 变成了悬垂引用（指向无效内存）。
            当有实例调用这个函数时，程序会尝试访问已经释放的内存，导致未定义行为。

        修复方案：
            直接返回 tx.payload 的切片。因为 tx 是 self 数据的引用，
            所以 &tx.payload[0..4] 的生命周期与 self 相同，是安全的。

            这里if let Some(tx) = self.txs.last() 
            右边的 self.txs.last() 返回的是对 Vec 中最后一个 Transaction 的引用

            这里的 tx 是一个引用类型 &Transaction。
            切片 &tx.payload[0..4] 也是一个引用类型 &str，
            作为栈上的一个胖指针，只有两个字段：指向堆上字符串数据的指针和字符串长度。
            所以是零拷贝的高效操作。

        底层细节：
            Rust 使用借用检查器（Borrow Checker）来跟踪引用的生命周期。
            当尝试返回指向局部变量的引用时，借用检查器会检测到该引用的生命周期短于函数调用者的生命周期，
            因此会报错以防止悬垂引用的产生。

        本质上：
            引用的生命周期绝对不能长于数据所有者的生命周期。皮之不存，毛将焉附。
         */
    }
    
}

// ❌ 陷阱 3: 部分移动 (Partial Move)
// 场景：在处理复杂结构体时，不小心把字段的所有权拆散了
fn check_partial_move() {
    let tx = Transaction {
        id: 101,
        payload: String::from("Mint 100 BTC"),
    };

    /*
    出错原因：
        let payload_ref = tx.payload; 将 payload 字段的所有权从 tx 结构体中移走了。
        此时 tx 变成了部分失效状态（Partially Moved），
        因此不能再作为一个整体（如 println!("{:?}", tx)）被使用。
    
    修复方案：
        只获取 payload 的引用而不获取所有权。
        Rust 允许结构体的部分字段被移动，但一旦某个字段被移动，
        整个结构体就不能再被完整使用，除非该字段实现了 Copy trait。
     
    底层细节：
        Drop Flag：Rust 使用 Drop Flag 来跟踪结构体中哪些字段已经被移动。
        当某个字段被移动时，编译器会更新 Drop Flag，标记该字段已无效。
        这样在调用 drop 时，编译器就知道不要对已移动的字段重复释放内存，避免 double free 错误。
     */

    // 修复：使用引用 &tx.payload，避免所有权移动 (Partial Move)
    let payload_ref = &tx.payload; 
    
    // 此时 tx 还在吗？还能打印整个 tx 吗？
    // 现在能够打印整个 tx 结构体，因为我们没有移动任何字段的所有权
    // 但是之前如果移动了 payload 字段的所有权，打印整个 tx 会报错
    println!("完整交易: {:?}", tx); // 取消注释看看会发生什么
    
    // 思考：tx.id 还在栈上吗？还能用吗？
    // 答案是可以的，因为 id 是 Copy 类型，没有被移动
    // 虽然 payload 被引用了，但 tx 结构体本身依然完整
    // 即使修改之前的代码为 let payload_ref = tx.payload; 也不会影响 id 字段的使用
    // 因为 id 字段在栈上，且是 Copy 类型
    println!("交易ID: {}", tx.id); 
}

pub fn run_experiments() {
    println!("--- S01 进阶: 内存深水区 ---");

    let mut pool = Mempool::new();
    pool.add(Transaction { id: 1, payload: String::from("Tx_A") });
    pool.add(Transaction { id: 2, payload: String::from("Tx_B") });

    // 1. 尝试修复 pop_first
    let first_tx = pool.pop_first(); 
    println!("打包交易: {:?}", first_tx);

    // 2. 尝试修复悬垂引用
    let preview = pool.get_latest_payload_preview();
    println!("最新交易预览: {}", preview);

    // 3. 部分移动实验
    check_partial_move();
}