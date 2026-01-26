// src/s02_abstraction/ex04_lifetimes.rs

#[derive(Debug)]
pub struct ConsensusConfig {
    pub chain_id: u64,
    pub magic_bytes: String,
}

// ==========================================
// ❌ 陷阱区域 1：结构体中的引用
// ==========================================

// 我们定义一个校验器，它持有一个指向配置的引用
// 编译器报错：expected named lifetime parameter
// 潜台词："你这个结构体里有个引用，万一结构体还活着，引用的数据先死了怎么办？"
// "你必须给我保证：Validator 活多久，这个引用就要能活多久。"
pub struct Validator {
    pub config: &ConsensusConfig, // 这里缺了一个生命周期标注
}

// ==========================================
// ❌ 陷阱区域 2：实现块中的生命周期
// ==========================================

// 即使你修复了上面，这里也会报错。
// 因为 impl 也是泛型的，你得告诉编译器这里的 'a 是啥。
impl Validator {
    // 构造函数
    // 注意：输入的是引用的 config，输出的是持有引用的 Validator
    // 它们之间的生命周期必须关联起来
    pub fn new(config: &ConsensusConfig) -> Validator {
        Validator { config }
    }

    // 验证逻辑
    pub fn validate_block(&self, block_chain_id: u64) -> bool {
        if block_chain_id == self.config.chain_id {
            println!("✅ Block valid for chain {}", self.config.chain_id);
            true
        } else {
            println!("❌ Invalid chain id: expected {}, got {}", 
                self.config.chain_id, block_chain_id);
            false
        }
    }
}

pub fn run() {
    println!("--- S02 Ex04: 生命周期 (Zero-Copy) ---");

    // 1. 全局配置 (Owner) - 它住在 main 函数的栈底，活得最久
    let config = ConsensusConfig {
        chain_id: 1024,
        magic_bytes: String::from("ZK_ROLLUP"),
    };

    // 2. 创建一个作用域
    {
        // 3. 借用配置创建校验器
        let v = Validator::new(&config);
        
        // 4. 验证
        v.validate_block(1024);
        v.validate_block(999);
        
    } // v 在这里销毁，但 config 依然活着，所以这是安全的
    
    println!("Config is still alive: {:?}", config);
}