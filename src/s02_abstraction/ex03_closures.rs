// src/s02_abstraction/ex03_closures.rs

struct Transaction {
    amount: u32,
}

pub fn run() {
    println!("--- S02 Ex03: 闭包与迭代器 ---");

    let txs = vec![
        Transaction { amount: 5 },
        Transaction { amount: 10 },
        Transaction { amount: 15 },
        Transaction { amount: 20 },
    ];

    // ❌ 任务 1：使用迭代器链式调用
    // 目标：筛选出 amount > 10 的交易，把金额 * 2，然后求和。
    // 请取消注释并填空：
    
    let total_reward: u32 = txs.iter()
        .filter(|tx| tx.amount > 10 )  // 填空：如何判断金额 > 10
        .map(|tx| tx.amount * 2 )     // 填空：如何把金额 * 2 (注意：tx 是引用)
        .sum();// ||里面是参数列表 外面是函数体
    /*
    链式调用解析：
    第一步：txs.iter()
        状态: 产生了一个迭代器。
        元素: &Transaction（因为这是一个不拿走所有权的引用迭代器）。
    第二步：.filter(|tx| tx.amount > 10)
        tx 是什么: 这里有一个细节。filter 为了不吃掉元素，它在这个元素基础上又借用了一次。
            所以这里的 tx 实际上是 &&Transaction（引用的引用）。
        发生了什么: Rust 能够自动解引用，所以 tx.amount 依然能读到数据。
        联系: 也就是“对于每一个流经此地的 tx，请帮我判断它是否大于 10”。
    第三步：.map(|tx| tx.amount * 2)
        tx 是什么: 经过 filter 筛选后，传给 map 的依然是原来的元素 &Transaction。
        发生了什么: 这里我们做了数学运算。
        联系: 这里的 tx 和上面 filter 里的 tx 不是同一个变量，它们只是碰巧都叫 tx（就像两个不同部门的临时工都叫“小王”）。
            它是 map 这个环节接收到的参数。
        产出: 这一步之后，流出来的不再是 Transaction 结构体了，而是 u32 数字（因为 amount * 2 是数字）。
    第四步：.sum()
        行为: 把流出来的所有 u32 数字加起来。
     */

    println!("Total Reward: {}", total_reward);
    

    // ❌ 任务 2：闭包的“捕获”特性 (Capturing)
    // 场景：筛选阈值不是写死的，而是由外部变量控制的
    let min_limit = 10;
    
    // 请写一个闭包赋给变量 `checker`
    // 这个闭包接收一个 &Transaction，如果 amount > min_limit 返回 true
    let checker = |tx: &Transaction| -> bool { tx.amount > min_limit };
    /*
    到底什么是“闭包”？它和函数有什么区别？
        你可能会问：“为什么要搞个怪模怪样的 |...|，直接传个普通函数不行吗？”

        闭包 (Closure) = 函数 + 环境 (Context) (去糖代码底层本质上就是 带有数据的结构体 + call 方法)

        普通函数像是外包团队，它只知道你传给它的参数，别的什么都不知道。
        闭包像是贴身秘书，它不仅知道参数，还能偷偷看到你在旁边定义了什么变量。
        这里的 min_limit 就是闭包“偷偷看到”的外部变量。
    
    总结闭包的作用：
        匿名便利: 很多时候逻辑很简单（比如 x > 10），专门起个名字写个 fn 太麻烦，用 |x|随手一写最方便。
        捕获环境: 可以读取在这个闭包被定义时，周围上下文里的变量（比如 min_limit）。
            这在作为参数传递给 filter 等高阶函数时极其重要，
            因为预定义的 filter 接口通常只允许传一个参数（就是元素本身），如果你想让它和外部变量做比较，
            只有闭包能做到。
     */

    // 使用 filter(checker)
    let count = txs.iter().filter(|tx| checker(tx)).count();
    println!("Tx count > {}: {}", min_limit, count);
    /*
    发生了什么？
        这里我们定义了一个闭包 checker，它能访问外部的 min_limit 变量。
        当我们把 checker 传给 filter 时，filter 会在每次迭代时调用 checker，
        并把当前的 &Transaction 传给它。
        checker 会使用传入的 tx 和外部的 min_limit 做比较，返回 true 或 false。
     */
}

    /*
    结合代码解释下面三个重要底层概念：
    1.惰性求值 (Lazy Evaluation)：
        迭代器链式调用其实并不会立刻执行计算，而是构建了一个“计算计划”。
        只有在最后调用 sum() 时，整个链条才会被触发，数据才会真正流动起来。
        这就是惰性求值：只有需要结果时才计算，节省资源。

    2.闭包的真面目 (The Struct Behind) —— 编译器到底干了什么？
        每个闭包在编译时都会被编译器转换成一个匿名结构体 (Anonymous Struct)，
        这个结构体会把闭包捕获的外部变量作为它的字段存储起来。
        同时，闭包的代码会被转换成该结构体的一个方法 (Method)。
        这样，当你调用闭包时，实际上是在调用这个结构体的方法，
        并且这个方法可以访问结构体的字段（也就是闭包捕获的变量）。

    3.零成本抽象 (Zero-Cost Abstraction) —— 为什么它比 C++ 还要快？
        Rust 的闭包和迭代器都是零成本抽象，这意味着它们在编译后不会引入额外的运行时开销。
        这是因为：
            - 迭代器链在编译时会被内联展开 (Inline Expansion)，
              编译器会把整个链条的逻辑直接嵌入到最终的机器码中，避免了函数调用的开销。
            - 闭包捕获的变量在编译时就被确定下来，编译器可以优化访问方式，
              避免了运行时的动态查找。
        结果是，使用闭包和迭代器的代码在性能上可以媲美手写的循环，甚至更优，因为编译器能做更多优化。
     */