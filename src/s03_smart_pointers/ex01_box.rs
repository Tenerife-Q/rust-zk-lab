// src/s03_smart_pointers/ex01_box.rs

// ==========================================
// 1. 定义链表节点 (递归类型)
// ==========================================

#[derive(Debug)]
struct Node {
    value: i32,
    // ❌ 陷阱：直接包含自己会导致 "infinite size"
    // next: Option<Node>, 
    
    // ✅ 修复方案：使用 Box 指针
    // Box 让我们在 Heap 上分配内存，而这里只存一个 8 字节的指针
    next: Option<Box<Node>>,
}

// ==========================================
// 2. 封装一个简单的链表管理器
// ==========================================
struct LinkedList {
    head: Option<Box<Node>>,
}

impl LinkedList {
    fn new() -> Self {
        LinkedList { head: None }
    }

    // 头插法 (Push Front)
    fn push(&mut self, value: i32) {
        let new_node = Box::new(Node {
            value,
            // ❌ 陷阱 2：所有权转移问题
            // 我们需要把旧的 head 拿出来，放到新节点的 next 里
            // self.head 是 Option<Box<Node>>，它拥有所有权
            // 直接写 self.head 会报错：cannot move out of borrowed content
            // 提示：你需要一个方法把 Option 里的东西 "偷" 出来，同时留下一个 None
            next: self.head.take(), // ✅ 关键：使用 take()
        });

        self.head = Some(new_node);
    }

    // 打印链表
    fn print(&self) {
        // current 是一个引用，指向 Box 里的 Node
        let mut current = &self.head;
        
        print!("List: ");
        while let Some(node) = current {
            print!("{} -> ", node.value);
            // 移动指针
            current = &node.next;
        }
        println!("None");
    }
}

pub fn run() {
    println!("--- S03 Ex01: Box 与 递归链表 ---");

    let mut list = LinkedList::new();
    
    // 插入数据： 3 -> 2 -> 1 -> None
    list.push(1);
    list.push(2);
    list.push(3);

    list.print();
    
    // 思考题：当 list 离开作用域时，内存是如何释放的？
    // 是会发生 stack overflow 还是正常释放？
}