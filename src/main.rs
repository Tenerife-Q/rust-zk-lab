// 声明专题模块
mod s01_ownership;
mod s02_traits;
mod s03_zk_foundations;

// 第一个实验：递归结构与内存确定性
struct Block {
    data: String,
    prev_hash: Option<String>,
    // 实验任务：取消下面一行的注释，观察编译器报错并利用 NotebookLM 分析原因
    // prev_block: Option<Block>, 
}

fn main() {
    println!("--- Rust 深度实验室已启动 ---");
    
    let block_data = String::from("Genesis Block");
    
    let _genesis = Block {
        data: block_data,
        prev_hash: None,
    };
    
    // 费曼思考：如果在这里再次打印 block_data，会发生什么？为什么？
    // println!("{}", block_data); 

    println!("状态检查：环境配置成功，可以开始实验。");
}
