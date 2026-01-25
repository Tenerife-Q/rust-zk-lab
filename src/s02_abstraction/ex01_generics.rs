// src/s02_abstraction/ex01_generics.rs

// ==========================================
// 1. 定义契约 (Trait) - 对应《The Book》第10章
// ==========================================

// 任何想要存入账本的数据，都必须能够生成摘要
// 定义一个 Summarizable 契约
pub trait Summarizable {
    fn summarize(&self) -> String;
}

// ==========================================
// 2. 定义具体类型
// ==========================================

#[derive(Debug)]
pub struct BitcoinTx {
    pub tx_id: String,
    pub amount: u64,
}

#[derive(Debug)]
pub struct EthereumTx {
    pub from: String,
    pub to: String,
    pub gas_limit: u64,
}

// 为具体类型实现契约 impl...for...
// 多态性：不同的结构体（BitcoinTx, EthereumTx）有完全不同的字段，但它们都穿上了 Summarizable 这层"马甲"。
// 对于 Ledger 来说，它看到的不再是具体的比特币或以太坊交易，而只是"一个能 Summarize 的东西"
impl Summarizable for BitcoinTx {
    fn summarize(&self) -> String {
        format!("BTC Tx: {} | Amt: {}", self.tx_id, self.amount)
    }
}

impl Summarizable for EthereumTx {
    fn summarize(&self) -> String {
        format!("ETH Tx: From {} To {} | Gas: {}", self.from, self.to, self.gas_limit)
    }
}

// ==========================================
// 3. 定义泛型结构体 (The Generic Box)
// ==========================================

// 这是一个通用的账本，T 代表它可以存某种类型的记录
// 注意：结构体定义时，通常不需要加过多的约束
pub struct Ledger<T> {
    pub name: String,
    pub records: Vec<T>,
}

// ==========================================
// 4. 实现泛型方法 (The Logic)
// ==========================================

// ❌ 陷阱区域：这里的 impl<T> 没有对 T 加任何限制
impl<T: Summarizable> Ledger<T> {// 这里相当于impl...for...，对 T 加了必须实现 Summarizable 的约束
    pub fn new(name: &str) -> Self {
        Ledger {
            name: String::from(name),
            records: Vec::new(),
        }
    }
    /*
    1.关联函数 (Associated Function)：注意参数列表里没有 self。
        这意味着这个函数不属于某个具体的实例（比如“这本账本”），而属于 Ledger 类型本身。
        类似于其他语言的 static method（静态方法）。
        用法：调用时使用双冒号 Ledger::new(...)。
    2.参数 name: &str：参数是一个字符串切片（引用）。
        这是 Rust 的惯例，传参用 &str 更灵活（无论是字面量 "book" 还是 String 都能传），内部想要拥有它时再转成 String。
    3.返回值 -> Self：返回值类型。
        Self 是一个别名，代表“当前正在实现的类型”，在这里就等同于 Ledger<T>。
        写 Self 的好处是以后如果改了结构体名字，不用到处改代码。
    4.实现细节 String::from(name)：将借来的字符串引用（只读）复制一份，变成自己拥有的 String 对象（放在堆上）。
     */




    pub fn add_record(&mut self, record: T) {
        self.records.push(record);
    }
    /*
    &mut self：这是核心关键点。
        这意味着当调用这个方法时（ledger.add_record(...)），编译器会检查你是否有权限修改这本账本。
        如果没有 mut，你就不能改变内部的 records 向量。
    record: T：这个参数接收一个类型为 T 的值。

    这里发生了所有权转移 (Move)。
        当你把一条记录传进来，外部变量就失去了对这条记录的所有权，
        它被“移动”进了 self.records 向量里保存起来。 
    */




    // ❌ 编译器会在这里拦截你：
    // 你试图调用 .summarize()，但编译器不知道 T 是什么。
    // 万一 T 是个整数 i32 呢？i32 可没有 summarize 方法。
    pub fn print_audit_report(&self) {
        println!("--- Audit Report: {} ---", self.name);
        // 这里for循环语法糖展开后类似于：
        // for i in 0..self.records.len() {
        //     let record = &self.records[i];
        for (i, record) in self.records.iter().enumerate() {
            // Error: no method named `summarize` found for type `&T`
            println!("Record #{}: {}", i, record.summarize()); 
        }
    }
    /*
    &self：使用的是不可变引用。
        逻辑解释：打印报告只需要“看”数据，不需要“改”数据。因此用只读权限，更安全，也允许在这个过程中其他人也能同时“看”。
        如果你尝试在这个函数里写 self.records.push(...)，编译器会报错。
        核心语法糖详解：for 循环与迭代器
        
    这一行代码 for (i, record) in self.records.iter().enumerate() 做了四件复杂的事情：

    1. .iter() (创建迭代器)：
        self.records 是一个 Vec<T>。调用 .iter() 会生成一个迭代器，它每次吐出的元素是 &T（对T的引用）。
        为什么是引用？ 因为我们不能在遍历的时候把数据“吃掉”（拿走所有权），我们只是要看看它。

    2. .enumerate() (装饰器模式)：
        这是一个迭代器适配器。它包装了原来的迭代器。
        原来的迭代器每次给出一个 item。
        enumerate 把它加工变成一个元组：(index, item)，也就是 (索引, 元素引用)。
        第一次产生 (0, &item0)，第二次产生 (1, &item1)，以此类推。
    
    3.  模式匹配 (Pattern Matching)：
        for (i, record) in ...
        这里直接把产出的元组 (索引, 元素引用) 解构赋值给了变量 i 和 record。
        i 拿到了索引（0, 1, 2...）。
        record 拿到了元素的引用（&BitcoinTx 或 &EthereumTx）。

    4.  多态调用：
        record.summarize()：因为开头那个 T: Summarizable 的约束，编译器确信这里的 record 身上一定有 summarize 方法。
    
    综上，这行代码实现了“遍历账本里的每一条记录，同时知道它是第几条”的功能。
     */
}

pub fn run() {
    println!("--- S02 Ex01: 泛型账本 ---");

    // 1. 创建一个比特币账本
    let mut btc_ledger = Ledger::new("Satoshi's Book");
    btc_ledger.add_record(BitcoinTx { 
        tx_id: String::from("0x123..."), 
        amount: 50 
    });

    // 2. 创建一个以太坊账本
    let mut eth_ledger = Ledger::new("Vitalik's Notebook");
    eth_ledger.add_record(EthereumTx { 
        from: String::from("Alice"), 
        to: String::from("Bob"), 
        gas_limit: 21000 
    });

    // 3. 打印报告 (一旦你修复了泛型约束，这里就能跑)
    btc_ledger.print_audit_report();
    eth_ledger.print_audit_report();
}