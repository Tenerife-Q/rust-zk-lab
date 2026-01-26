// src/s02_abstraction/mod.rs

// 声明子模块
pub mod ex01_generics;
pub mod ex02_trait_objects;
pub mod ex03_closures;
pub mod ex04_lifetimes;

use std::io;

pub fn run_experiments() {
    loop {
        println!("\n--- 🧬 S02 抽象与契约 (Abstraction) ---");
        println!("1. 泛型与 Trait (Ledger System)");
        println!("2. Trait 对象 (Multi-Asset Wallet)");
        println!("3. 闭包与迭代器 (Tx Filter)");
        println!("4. 生命周期 (Zero-Copy Validator)");
        println!("0. 返回主菜单");
        println!("请输入练习编号:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("读取失败");

        match input.trim() {
            "1" => ex01_generics::run(),
            "2" => ex02_trait_objects::run(),
            "3" => ex03_closures::run(),
            "4" => ex04_lifetimes::run(),
            "0" => break,
            _ => println!("❌ 无效选择，请重试"),
        }
    }
}