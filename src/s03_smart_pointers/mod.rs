pub mod ex01_box;
pub mod ex02_rc;      
pub mod ex03_refcell;

use std::io;

pub fn run_experiments() {
    loop {
        println!("\n--- 🧠 S03 智能指针 (Smart Pointers) ---");
        println!("1. Box与递归类型 (Simple Blockchain)");
        println!("2. Rc 共享所有权 (DAG)");
        println!("3. RefCell 内部可变性");
        println!("0. 返回主菜单");
        println!("请输入练习编号:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("读取失败");

        match input.trim() {
            "1" => ex01_box::run(),
            "0" => break,
            "2" => ex02_rc::run(),
            "3" => ex03_refcell::run(),
            _ => println!("❌ 无效选择"),
        }
    }
}