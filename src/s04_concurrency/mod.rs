pub mod ex01_thread;
// pub mod ex02_sync;    // 待解锁
// pub mod ex03_channel; // 待解锁

use std::io;

pub fn run_experiments() {
    loop {
        println!("\n--- ⚡ S04 并发安全性 (Concurrency) ---");
        println!("1. 线程基础与 Move (Mining Simulator)");
        println!("0. 返回主菜单");
        println!("请输入练习编号:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("读取失败");

        match input.trim() {
            "1" => ex01_thread::run(),
            "0" => break,
            _ => println!("❌ 无效选择"),
        }
    }
}