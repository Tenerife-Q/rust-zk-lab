// src/s01_memory.rs

// 引入标准库，用于打印地址观察内存
use std::mem;

// ==========================================
// 1. 定义数据结构 (Struct) - 对应《The Book》第5章
// ==========================================

// 这是一个模拟的链上账户
// 考点：String 在堆上，u64 在栈上，Account 实例本身的布局取决于它在哪
#[derive(Debug)] // 让结构体可以被 {:?} 打印
struct Account {
    id: u64,
    owner: String,
    balance: u64,
}

// ==========================================
// 2. 定义逻辑分支 (Enum) - 对应《The Book》第6章
// ==========================================

// 交易类型
#[derive(Debug)]
enum Transaction {
    Deposit(u64),             // 存款：只包含金额
    Withdraw(u64),            // 取款：只包含金额
    Transfer { to: String, amount: u64 }, // 转账：包含目标地址和金额（匿名结构体风格）
}

// ==========================================
// 3. 实现行为 (Impl) - 综合运用
// ==========================================

impl Account {
    // 构造函数：创建一个新账户
    fn new(id: u64, owner: String) -> Account {
        Account {
            id,
            owner,      // 所有权从参数转移进结构体
            balance: 0, // 初始余额为 0
        }
    }

    // 打印账户详情
    // 注意：这里使用的是 &self (不可变引用)
    fn print_info(&self) {
        println!("Account ID: {}, Owner: {}, Balance: {}", self.id, self.owner, self.balance);
    }

    // 处理交易
    // ❌ 错误点预警：注意这里的 self 写法，对应第 4 章的方法语法
    // 需要可变引用 &mut self 来修改余额, 还要枚举中每一个分支都处理到
    fn process_tx(&mut self, tx: Transaction) {
        match tx {
            Transaction::Deposit(amount) => {
                self.balance += amount;
                println!("存入 {} 成功。", amount);
            }
            Transaction::Withdraw(amount) => {
                if self.balance >= amount {
                    self.balance -= amount;
                    println!("取款 {} 成功。", amount);
                } else {
                    println!("余额不足！");
                }
            }
            // ❌ 埋点 1 (第6章): 模式匹配必须是穷尽的 (Exhaustive)
            // 这里故意漏掉了 Transfer 类型，编译器会拦截你
            Transaction::Transfer {to, amount} => {
                if self.balance >= amount {
                    self.balance -= amount;
                    println!("转账 {} 给 {} 成功。", amount, to);
                } else {
                    println!("余额不足，无法转账！");
                }
            }
        }
    }

    // 销毁账户并返回所有者名字
    // ❌ 埋点 2 (第4章): 此方法获取了 self 的所有权 self是什么 
    // self 是 self: Self 的语法糖 Self 代表当前类型Account
    // self 代表调用该方法的实例本身
    fn close_account(self) -> String {
        println!("账户 {} 已销毁。", self.id);
        self.owner // 返回 String，所有权移出
    }
}

pub fn run_experiments() {
    println!("--- 综合实验: 账户与交易系统 ---");

    // 1. 创建账户
    // ❌ 埋点 3 (第3章): 变量默认是不可变的，但后续我们要修改余额
    let mut my_account = Account::new(1, String::from("Satoshi"));
    
    // 打印初始状态
    my_account.print_info();

    // 2. 模拟交易
    let tx1 = Transaction::Deposit(100);
    let tx2 = Transaction::Withdraw(50);
    let tx3 = Transaction::Transfer { 
        to: String::from("Vitalik"), 
        amount: 20 
    };

    // 执行交易
    // 注意：如果你修复了 my_account 的 mut 问题，这里需要传入可变引用吗？
    // Rust 的方法调用语法糖会自动处理引用，但定义处必须是 &mut self
    my_account.process_tx(tx1); 
    my_account.process_tx(tx2);
    my_account.process_tx(tx3); // 如果修复了埋点1，这里应该能跑

    // 3. 所有权与内存布局观察
    // 这是一个非常重要的费曼考点
    // 以下分别是打印栈和堆地址的示例
    println!("Stack address of account: {:p}", &my_account);// {:p}是打印指针地址的格式化符号 &my_account 是栈地址
    println!("Heap address of owner name: {:p}", my_account.owner.as_ptr());// owner.as_ptr() 是堆地址

    // 4. 销毁账户
    let owner_name = my_account.close_account();
    println!("已取回所有者名字: {}", owner_name);

    // 5. 再次尝试打印？
    // my_account.print_info(); // 取消注释这行会报错：value used after move
}