// src/s03_smart_pointers/ex01_box.rs

// ==========================================
// 1. 定义链表节点 (递归类型)
// ==========================================

#[derive(Debug)]
struct Node {
    value: i32,
    // ❌ 陷阱：直接包含自己会导致 "infinite size"
    // next: Option<Node>, 

    /*
    逻辑：链表由节点组成，每个节点包含数据 value 和指向下一个节点的“指针” next。
    Rust 特性：
        Option：用来表达“有”或“无”（即空指针 null 的安全替代品）。None 代表链表结束。
        Box：必须使用 Box。
            如果直接写 next: Option<Node>，
            Rust 编译器无法确定 Node 的大小（因为 Node 里包含 Node，无穷递归），
            导致编译错误 "infinite size"。Box<Node> 的大小是固定的（在 64 位机器上是 8 字节指针），
            从而打破了递归大小的无限循环。
        内存布局：Stack (HEAD指针) -> Heap (Node 1) -> Heap (Node 2) ...
     */

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
/*
这是一个封装层。用户不需要直接操作 Node，而是操作 LinkedList。它只持有链表的头指针 head。
head 是一个 Option<Box<Node>>，表示链表可能为空（None）或指向第一个节点（Some(Box<Node>)）。
*/

impl LinkedList {
    fn new() -> Self {
        LinkedList { head: None }
    }

    // 头插法 (Push Front)：把一个新数据塞到链表的最前面（头插法）。
    fn push(&mut self, value: i32) {
        let new_node = Box::new(Node {
            value,
            // ❌ 陷阱 2：所有权转移问题
            // 我们需要把旧的 head 拿出来，放到新节点的 next 里
            // self.head 是 Option<Box<Node>>，它拥有所有权
            // 直接写 self.head 会报错：cannot move out of borrowed content
            // push 持有的是 &mut self（可变借用）。你不能从借来的容器中直接“移走” (Move) 字段的所有权。
            // 数据完整性：如果移走 head，struct 就留下了一个“空洞”（未初始化内存），Rust 严禁这种状态存在。
            // next: self.head, // ❌ 错误写法
            // 提示：你需要一个方法把 Option 里的东西 "偷" 出来，同时留下一个 None
            next: self.head.take(), // ✅ 关键：使用 take()
            // take() 的本质作用：
            //  原子置换：它读取 Option 中的值，瞬间填入 None，并返回原值。
            //  结果：所有权成功转移到了 next，同时 self.head 变成了 None（一个合法的空状态），
            //  维持了结构体的完整性
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

/*
    默认 Drop (递归陷阱)：
        Rust 自动生成的 Drop 是递归的：Drop(A) -> Drop(B) -> Drop(C)...
        每一个节点的释放都需要压栈 (Stack Frame)。
    后果：当链表过长（如 10 万个节点）时，会导致 Stack Overflow (栈溢出)。
*/
// ✅ 修复方案：手动实现 Drop 以避免递归析构导致的栈溢出
impl Drop for LinkedList {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        // while let 语法：当 cur_link 是 Some 时循环
        // 每次取出一个节点，并将其 next 指针 take() 走
        // 这样每个节点在离开作用域时不会递归 drop
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
            // boxed_node 在这里离开作用域并被丢弃
            // 但因为它的 `next` 已经被 take() 走了（变成 None），
            // 所以不会触发递归 drop
        }
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
    // 答案：由于我们手动实现了 Drop，链表节点会逐个被释放，避免了递归析构导致的栈溢出。
    // 因此，内存会被正常释放，而不会发生 stack overflow。



/*
    异常安全性
        误区 1：悬空指针？
            否。被移走的数据在堆上活得好好的，没有被释放，所以不算悬空。
        误区 2：生命周期不一致？
            否。数据没有死，只是我们无法通过被借用的 self 去移动它。
        真相：异常安全性 (Exception Safety)
            假设 Rust 允许直接移走 head 而不填 None。
            如果代码在移走之后、填入新节点之前发生了 Panic（崩溃），Rust 会开始清理栈变量（Unwinding）。
            此时 drop 会试图释放 self.head。
            如果 head 里的东西已经被移走了（变成了垃圾数据），
            drop 就会造成 Double Free（双重释放） 或 Undefined Behavior（未定义行为）。
        结论：take() 保证了任何时刻（包括 Panic 发生时），self.head 要么是有值的，要么是 None，绝不会是“被掏空”的非法状态。
 */
}

