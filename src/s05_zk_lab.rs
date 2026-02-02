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

        // 第一步：把所有数据变成叶子节点 (S01 Iterator)
        let nodes: Vec<Box<Node>> = data.iter()
            .map(|d| Box::new(Node::new_leaf(d)))
            .collect();

        // 第二步：递归构建树
        let root = Self::build_recursive(nodes);

        MerkleTree {
            root: Some(root),
            leaves: data,
        }
    }

    // 递归构建函数 (核心逻辑)
    // 输入：一排节点
    // 输出：这排节点归约后的唯一根节点
    fn build_recursive(mut nodes: Vec<Box<Node>>) -> Box<Node> {
        // 递归基准条件 (Base Case)
        if nodes.len() == 1 {
            return nodes.pop().unwrap(); // 拿出最后一个，返回
        }

        // 如果节点数是奇数，复制最后一个节点凑成偶数 (Bitcoin 的做法)
        if nodes.len() % 2 != 0 {
            let last = nodes.last().unwrap().clone();
            nodes.push(last);
        }

        let mut next_level = Vec::new();

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
        let mut iter = nodes.into_iter();
        while let Some(left) = iter.next() {
            // 由于上面已保证节点数为偶数，这里一定能取到 right
            let right = iter.next().expect("node count should be even");
            let parent = Node::new_internal(left, right);
            next_level.push(Box::new(parent));
        }
        
        // --- 你的代码区域 End ---

        // 递归调用：构建上一层
        Self::build_recursive(next_level)
    }

    pub fn root_hash(&self) -> String {
        match &self.root {
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