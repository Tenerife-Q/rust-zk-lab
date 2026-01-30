// src/main.rs

mod s01_memory;
mod s02_abstraction; 
mod s03_smart_pointers;
mod s04_concurrency;

use std::io;

fn main() {
    loop {
        println!("\n=============================================");
        println!("    🦀 Rust 工程化复习实验室 (v4.0)    ");
        println!("=============================================");
        println!("1. S01: 内存基本法 (Memory)");
        println!("2. S02: 抽象与契约 (Traits) [已解锁]");
        println!("3. S03: 智能指针 (Smart Pointers) [已解锁]");
        println!("4. S04: 并发安全性 (Concurrency) [已解锁]");
        println!("0. 退出系统");
        println!("请选择板块:");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("读取失败");

        match choice.trim() {
            "1" => s01_memory::run_experiments(),
            "2" => s02_abstraction::run_experiments(), // ✅ 这里接入 S02
            "3" => s03_smart_pointers::run_experiments(), // ✅ 这里接入 S03
            "4" => s04_concurrency::run_experiments(), // ✅ 这里接入 S04
            "0" => {
                println!("👋 再见!");
                break;
            },
            _ => println!("❌ 无效选择"),
        }
    }
}