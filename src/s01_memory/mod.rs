// src/s01_memory/mod.rs

// 声明子模块（对应文件名）
pub mod ex01_basic;
pub mod ex02_advanced;

use std::io;

// S01 板块的二级菜单
pub fn run_experiments() {
    loop {
        println!("\n--- 🧠 S01 内存基本法 (Memory) ---");
        println!("1. 基础篇：Account 结构体与布局");
        println!("2. 进阶篇：Mempool、所有权陷阱 (NEW!)");
        println!("0. 返回主菜单");
        println!("请输入练习编号:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("读取失败");

        match input.trim() {
            "1" => ex01_basic::run(),     // 运行你刚才写的 Account
            "2" => ex02_advanced::run(),  // 运行新的 Mempool 题目
            "0" => break,                 // 跳出循环，返回 main
            _ => println!("❌ 无效选择，请重试"),
        }
    }
}