// src/s02_abstraction/ex02_trait_objects.rs

// 1. 定义资产行为
pub trait Asset {
    fn display(&self) -> String;
}

// 2. 定义两种不同的资产
struct Token {
    symbol: String,
    amount: u64,
}

struct NFT {
    id: u64,
    url: String,
}

impl Asset for Token {
    fn display(&self) -> String {
        format!("Token: {} (Amt: {})", self.symbol, self.amount)
    }
}

impl Asset for NFT {
    fn display(&self) -> String {
        format!("NFT #{} (Url: {})", self.id, self.url)
    }
}

// 3. 混合钱包 (The Mixed Bag)
struct Wallet {
    // ❌ 错误做法：使用泛型 T
    // Vec<T> 意味着：虽然 T 可以是任何实现了 Asset 的类型
    // 但一旦确定了是 Token，整个 Vec 就只能装 Token，不能混入 NFT
    // assets: Vec<T>, 
    
    // ✅ 正确做法：使用 Trait 对象
    // 这里的 Box<dyn Asset> 翻译成人话就是：
    // "我不知道它具体多大，但我知道它在堆上，且它有个指针指向 Asset 的方法表"
    assets: Vec<Box<dyn Asset>>,
    /*
    物理形态：
    这里的 Box<dyn Asset> 在内存中是一个 胖指针 (Fat Pointer)。
    普通指针只占 8字节（64位系统），只存数据地址。
    胖指针占 16字节 = 对象数据的内存地址 + 对应类型的虚函数表地址 (vptr)。
        前 8字节：指向堆上实际数据的地址（比如那是具体的 Token 数据,即struct定义的内存位置）。
        后 8字节：指向一个虚函数表 (vtable) 的地址。这个表里记录了 Token 的 display 方法在哪
    虚函数表 (vtable)：
        VTable 是在 编译时 (Compile Time) 生成的，存储在二进制文件的 只读数据段 (RODATA / .text) 中。
        vtable 是编译器为每个实现了 Asset 契约的具体类型（Token, NFT）生成的一张方法指针表。
        表里存的都是函数指针，告诉系统该如何调用这个类型实现的 Asset 方法。
        比如 Token 的 vtable 可能长这样：
            [
            drop_in_place,  // 析构函数指针（怎么销毁这个 Token）
            size,           // Token 的大小 (24 字节)
            align,          // Token 的对齐要求 (8 字节)
            display,        // Token::display 函数的实际内存地址 <--- 关键！
            ...             // 其他实现了 Asset 的方法指针
            ]

    A. dyn (Dynamic) —— 动态类型
        T: Asset (泛型) 是静态的。编译器在编译时就知道 T 是什么，并把代码复制一份。
        dyn Asset 是动态的。
            它告诉编译器："这里放的是一种实现了 Asset 契约的东西，具体是啥？我不到程序跑起来那一刻我都不知道。"
        代价：因为不知道具体类型，编译器就不知道该给它留多大的内存空间（Token 占24字节，NFT 占32字节，完全不同）。
        这种类型叫 "Unsized" (不定长类型) —— 在 Rust 中，不定长的东西不能直接放在栈上或是 Vec 里。
    B. Box (盒子) —— 强制定长
        为了能把这些"高矮胖瘦各不相同"的数据放进同一个 Vec 数组里，我们必须把它们统一包装。
        Box 就是一个智能指针，指向堆内存。
        不管里面的数据（Token/NFT）有多大，指针本身的大小是固定的（在64位系统上是8字节）。
        所以 Vec<Box<dyn Asset>> 实际上存的是一排整齐划一的指针。
        
    这样一来，我们就能在运行时动态地往钱包里添加各种不同类型的资产了！
     */
}

impl Wallet {
    fn new() -> Wallet {
        Wallet { assets: Vec::new() }
    }

    // 这里的参数为什么必须是 Box<dyn Asset>？
    // 因为 dyn Asset 是一个"不定长类型"(Unsized)，不能直接放在栈上传递
    fn add_asset(&mut self, asset: Box<dyn Asset>) {
        self.assets.push(asset);
    }
    /*
    入参：这个函数接收所有权。当你传入 Box::new(Token{...}) 时，系统发生了隐式转换（Coercion）：
        从具体的 Box<Token> 转换成了抽象的 Box<dyn Asset>。
    转换过程：在这个转换瞬间，Rust 编译器生成了那个能够指向 Token 方法的 vtable 指针，
        并把它和数据指针打包在了一起。
     */


    fn show_portfolio(&self) {
        println!("--- Wallet Portfolio ---");
        for (i, item) in self.assets.iter().enumerate() {
            // 这里发生了 "动态分发" (Dynamic Dispatch)
            // 运行时查表找到对应的 display 方法
            println!("Item {}: {}", i, item.display());
        }
    }
    /*
    发生了什么？
        当循环跑到 item.display() 时，CPU 拿到的是一个不知道具体类型的胖指针。
        CPU 执行动作：“这有个胖指针。我看下它的第二个部分（vtable 指针），
            去那个表里查一下 display 函数在内存的哪个位置？哦，在地址 0x1234。好，跳转到 0x1234 去执行。”
        这就是“动态”：在编译期间，编译器根本不知道会跳到哪里去，只有运行时才知道。

     */
}

pub fn run() {
    println!("--- S02 Ex02: 混合钱包 (Trait Objects) ---");

    let mut my_wallet = Wallet::new();

    // 1. 创建 Token
    let t1 = Token { symbol: String::from("USDT"), amount: 100 };
    
    // 2. 创建 NFT
    let n1 = NFT { id: 8888, url: String::from("ipfs://...") };

    // ❌ 编译报错区域
    // 任务：请修复下面这两行代码
    // 提示：add_asset 需要的是一个“盒子”(Box)，但你传的是原始结构体
    // my_wallet.add_asset(t1);
    // my_wallet.add_asset(n1);
    // ✅ 修复后的代码：将具体的结构体装进 Box 指针中
    my_wallet.add_asset(Box::new(t1));// 传入 Box<dyn Asset>
    my_wallet.add_asset(Box::new(n1));// Box::new 会自动把 Token/NFT 转换成 dyn Asset

    // 3. 显示钱包内容
    my_wallet.show_portfolio();
}