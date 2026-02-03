// src/s05_zk_lab.rs
// use std::fmt;

// 引入一个简易的哈希模拟函数（在真实项目中我们会用 sha2/keccak）
// 这里为了不引入外部 crate，我们用标准库模拟一个 "Hash"
fn mock_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// ==========================================
// 1. 定义 Merkle 节点 (递归结构) - S03 Box
// ==========================================
#[derive(Debug, Clone)]
struct Node {
    hash: String,
    // 左孩子和右孩子。如果是叶子节点 (Leaf)，这两个都是 None
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    // 创建叶子节点
    fn new_leaf(data: &str) -> Self {
        Node {
            hash: mock_hash(data),
            left: None,
            right: None,
        }
    }

    // 创建中间节点
    fn new_internal(left: Box<Node>, right: Box<Node>) -> Self {
        // ❌ 任务 1：计算父节点的哈希
        // 规则：parent_hash = hash(left.hash + right.hash)
        // 提示：使用 format! 拼接字符串，然后调用 mock_hash
        let combined_data = format!("{}{}", left.hash, right.hash); 
        let new_hash = mock_hash(&combined_data);

        /*
        参数 left: Box<Node>：没有 &。说明这个函数是个强盗，它会把传入的子节点的所有权直接抢过来。
        left: Some(left)：抢过来的所有权，直接装进自己的口袋（Option 字段）。
        这就是为什么 build_recursive 可以不用 clone 的原因：
            子节点自愿把自己“献祭”给了父节点，成为了父节点的一部分。
         */
        Node {
            hash: new_hash,
            left: Some(left),   // 所有权移入
            right: Some(right), // 所有权移入
        }
    }
}

// ==========================================
// 2. Merkle Tree 结构体
// ==========================================
pub struct MerkleTree {
    root: Option<Box<Node>>,
    pub leaves: Vec<String>, // 保存原始数据，便于验证
}

impl MerkleTree {
    pub fn new(data: Vec<String>) -> Self {
        if data.is_empty() {
            return MerkleTree { root: None, leaves: vec![] };
        }
        /*
        在其他语言可能会因为空数组导致数组越界 (IndexOutOfBounds) 或者递归死循环。
        Rust 这里做了一个优雅的防御：如果给了空数据，我直接还你一棵“空树”（root: None）。
        vec![] 宏创建了一个空的 Vector
         */

        // 第一步：把所有数据变成叶子节点 (S01 Iterator)
        let nodes: Vec<Box<Node>> = data.iter()//
            .map(|d| Box::new(Node::new_leaf(d)))
            .collect();
        /*
        data.iter().map(...).collect() (链式调用)：
        .iter()：借用 data 里的元素。
        .map(|d| ...)：闭包 (Closure)。把每个字符串 d 变成一个 Box::new(Node::new_leaf(d))。
        .collect()：这是 Rust 迭代器最强的地方。它会自动根据左边的类型标注 Vec<Box<Node>>，
            把 map 产出的元素收集成一个 Vector。
        
        Box::new(...)：
        Box 是堆内存分配。因为 Node 是递归结构，大小不定，如果不装在箱子（指针）里，编译器无法确定大小。
         */

        // 第二步：递归构建树 
        // 这是最外层调用
        let root = Self::build_recursive(nodes);
        // 调用关联函数 (Associated Function)，传入节点列表，返回根节点
        // 这里把刚才打包好的那箱 nodes（所有权）直接扔给了 build_recursive。
        // 所有权转移：在这行之后，new 函数里的 nodes 变量就不能用了。它归 build_recursive 管了。


        MerkleTree {
            root: Some(root),
            leaves: data,// 因为之前使用的是 data.iter()，data 仍然拥有所有权，可以直接用
        }

        /*
        总结 new 做了什么？
            它是一个完美的转换器：
            输入 Vec<String> （一堆生肉）
            --> 映射转化为 Vec<Box<Node>> 赋给变量nodes（做成香肠）
            --> 调用build_recursive递归压缩为 Box<Node> 返回给root（打包成礼盒）
            --> 输出 MerkleTree 对象 （发货）
         */
    }

    // 递归构建函数 (核心逻辑)
    // 输入：一排节点
    // 输出：这排节点归约后的唯一根节点
    fn build_recursive(mut nodes: Vec<Box<Node>>) -> Box<Node> {
        // 递归基准条件 (Base Case)
        if nodes.len() == 1 {
            return nodes.pop().unwrap(); // 拿出最后一个，返回
        }
        /*
        pop()：从 nodes 列表末尾弹出一个元素。返回 Option<Box<Node>>。
        unwrap()：因为我们已经确认 len() == 1，所以这里一定有值。直接拆包拿出来返回。
         */

        // 如果节点数是奇数，复制最后一个节点凑成偶数 (Bitcoin 的做法)
        if nodes.len() % 2 != 0 {
            let last = nodes.last().unwrap().clone();
            nodes.push(last);
        }
        /*
        nodes.last()：借用看一下最后一个元素（不拿走）。
        unwrap()：确认有值，拆包拿出来。将会得到 &Box<Node>。
        .clone()：深拷贝。这里必须克隆，因为我们要把它复制一份追加到队尾。
        nodes.push(last)：把克隆体塞进列表。如果不加 mut 关键字，这行就会报错。
         */

        let mut next_level = Vec::new();// 保存上一层节点的容器

        // ❌ 任务 2：成对处理节点，生成上一层
        // 提示：使用 chunks(2) 迭代，每次拿两个节点 left 和 right
        // 注意：chunks 给的是引用，你需要处理所有权问题 (clone 或 重新设计)
        // 更简单的做法：使用 Vec::drain 或 windows，或者直接用 for 循环 + index
        
        // 建议方案：使用 while 循环从 nodes 里弹出
        // (这是对 S01 Move语义 和 S03 Box 的综合考验)
        
        // --- 你的代码区域 Start ---
        // 伪代码提示：
        // 遍历 nodes (步长为2):
        //    left = nodes[i]
        //    right = nodes[i+1]
        //    parent = Node::new_internal(left, right)
        //    next_level.push(parent)
        
        /* 
        // 原始实现（虽然可行，但因为 chunks 只给引用，导致必须 clone Box<Node> 即深拷贝整棵子树，效率较低）：
        for chunk in nodes.chunks(2) {
             let left = chunk[0].clone();
             let right = chunk[1].clone();
             next_level.push(Box::new(Node::new_internal(left, right)));
        }
        */

        // 优化方案：把 nodes 的所有权转移给迭代器，避免 clone 整个子树
        // 注意：into_iter 会按原顺序逐个产出节点，保证 Merkle 树顺序一致
        // into_iter 不是借用，会消耗 nodes，之后不能再用它，后面新一轮就用 next_level 了
        let mut iter = nodes.into_iter();
        // 一次循环调两次next(),先后取出 left 和 right
        // 第一次拿 left。如果拿不到（None），说明传送带空了，循环结束 (while let）。
        while let Some(left) = iter.next() {
            // 由于上面已保证节点数为偶数，这里一定能取到 right
            // 第二次拿 right。因为前面补齐了偶数，所以这里必然有值。
            // expect()：如果取不到就 panic，提示“节点数应该是偶数”。但是这里不会发生。
            let right = iter.next().expect("node count should be even");
            // 调用 new_internal 创建父节点 
            // 注意：left 和 right 的所有权被转移进 new_internal
            // new_internal 将左右两棵子树合并，返回一个 Node 类型
            let parent = Node::new_internal(left, right);
            next_level.push(Box::new(parent));
        }
        
        // --- 你的代码区域 End ---

        // 递归调用：构建上一层
        Self::build_recursive(next_level)

        /*
        第一层：输入 4 个，产出 [P1, P2] -> 扔给自己。
        第二层：输入 2 个，产出 [Root] -> 扔给自己。
        第三层：输入 1 个 -> 触发 [阶段 1]，直接返回 Root。
        砰！砰！砰！ 递归栈层层弹回，最终最外层函数拿到了那个 Root。
        
        总结 build_recursive
            它是一个不需要垃圾回收的内存机器。
            通过 into_iter 和 Option 配合，它像贪吃蛇一样吞噬掉上一层的所有节点，
            将它们的所有权转移给下一层，直到最后只剩一个头。没有任何内存泄漏，也没有任何不必要的复制。
         */

    }

    pub fn root_hash(&self) -> String {
        match &self.root {
            // node.hash 是 String 类型。
            // .clone()：因为我要返回一个 String 给外部，而我手里只有一个借来的引用。
            //  我不能把别人的东西送人，所以我必须复印一份（深拷贝字符串内容）送出去
            Some(node) => node.hash.clone(),
            None => String::from(""),
        }
    }
}

pub fn run() {
    println!("--- S05: ZK Lab (Merkle Tree) ---");

    // 模拟区块链交易
    let transactions = vec![
        String::from("Tx1: Alice->Bob"),
        String::from("Tx2: Bob->Charlie"),
        String::from("Tx3: Charlie->Dave"),
    ];

    println!("Building Merkle Tree for {} transactions...", transactions.len());
    let tree = MerkleTree::new(transactions);

    /*
    交接：这是最关键的一行。
        调用了 new。
        所有权移交：transactions 变量被传入 new。
        从此以后，run 函数里再也不能使用 transactions 这个变量了！
        它已经属于 tree 对象内部了（变成了 tree.leaves）。
    内部发生的事：
        mock_hash 突突突地生成指纹。
        build_recursive 呼啦啦地递归构建。
        最终，所有的计算瞬间完成，返回一个封装好的 tree 对象。
     */

    println!("Root Hash: {}", tree.root_hash());

    // ❌ 任务 3：手动验证 (费曼前置)
    // 请画出这棵树的结构（Tx3 被复制了一次，所以是 4 个叶子）
    // 计算路径：
    // H(Root) = H( H(Tx1+Tx2) + H(Tx3+Tx3) )
    // 请运行代码，看输出是否符合你的预期。
    println!("\n--- Manual Verification ---");
    // transactions 所有权移进去了，从 tree.leaves 拿
    let h1 = mock_hash(&tree.leaves[0]);
    let h2 = mock_hash(&tree.leaves[1]);
    let h3 = mock_hash(&tree.leaves[2]);
    let h4 = h3.clone(); // 奇数个，复制最后一个

    let p1 = mock_hash(&format!("{}{}", h1, h2));
    let p2 = mock_hash(&format!("{}{}", h3, h4));
    let expected_root = mock_hash(&format!("{}{}", p1, p2));

    println!("Manual Calc: {}", expected_root);
    
    // transactions 所有权已移交给 tree，所以这里从 tree.leaves 取数据验证
    if tree.root_hash() == expected_root {
        println!("✅ Verification Success!");
    } else {
        println!("❌ Verification Failed!");
    }
}