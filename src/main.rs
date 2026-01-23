// src/main.rs

// 这里的 s01_memory 会自动去寻找 s01_memory/mod.rs
mod s01_memory;
// mod s02_abstraction; 

use std::io;

fn main() {
    loop {
        println!("\n=============================================");
        println!("    🦀 Rust 工程化复习实验室 (v3.0)    ");
        println!("=============================================");
        println!("1. S01: 内存基本法 (Memory)");
        println!("2. S02: 抽象与契约 (Traits) [锁定]");
        println!("0. 退出系统");
        println!("请选择板块:");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("读取失败");

        match choice.trim() {
            "1" => s01_memory::run_experiments(), // 进入二级菜单
            "0" => {
                println!("👋 再见!");
                break;
            },
            _ => println!("❌ 无效选择"),
        }
    }
}