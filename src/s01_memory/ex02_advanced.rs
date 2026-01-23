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
        None // 临时占位，修复后删掉这行
    }

    // ❌ 陷阱 2: 悬垂引用 (Dangling Reference)
    // 场景：我们想获取最新交易的 payload 的切片用于打印日志
    // 这是一个非常隐蔽的错误，请仔细阅读报错
    /* fn get_latest_payload_preview(&self) -> &str {
        if let Some(tx) = self.txs.last() {
            // 假设我们要对 payload 做一些处理（比如截取前4位）再返回
            let temp_str = tx.payload.clone(); // 克隆了一份数据到局部变量
            return &temp_str[0..4];            // 返回局部变量的引用
        }
        "Empty"
    }
    */
}

// ❌ 陷阱 3: 部分移动 (Partial Move)
// 场景：在处理复杂结构体时，不小心把字段的所有权拆散了
fn check_partial_move() {
    let tx = Transaction {
        id: 101,
        payload: String::from("Mint 100 BTC"),
    };

    // 我们把 payload 的所有权移动给了 log_payload
    let payload_ref = tx.payload; 
    
    // 此时 tx 还在吗？还能打印整个 tx 吗？
    // println!("完整交易: {:?}", tx); // 取消注释看看会发生什么
    
    // 思考：tx.id 还在栈上吗？还能用吗？
    println!("交易ID: {}", tx.id); 
}

pub fn run_experiments() {
    println!("--- S01 进阶: 内存深水区 ---");

    let mut pool = Mempool::new();
    pool.add(Transaction { id: 1, payload: String::from("Tx_A") });
    pool.add(Transaction { id: 2, payload: String::from("Tx_B") });

    // 1. 尝试修复 pop_first
    // let first_tx = pool.pop_first(); 
    // println!("打包交易: {:?}", first_tx);

    // 2. 尝试修复悬垂引用
    // let preview = pool.get_latest_payload_preview();
    // println!("最新交易预览: {}", preview);

    // 3. 部分移动实验
    check_partial_move();
}