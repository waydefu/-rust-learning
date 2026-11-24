# 第03章｜把程式拆成資料、規則與選擇

本章是後續所有章節的語法地基。核心不是記關鍵字，而是理解資料如何被命名、限制、運算及流轉。

## 一、為什麼需要變數與型別？

CPU 可處理位元，但同一段位元可以解釋成數字、字元或地址。程式必須聲明「這段資料如何被解讀、哪些運算合法」，這就是型別系統的重要任務。

比喻：箱子貼「公升」或「公斤」標籤。兩個值都可寫 5，卻不能在沒有轉換規則下當成相同概念。型別幫你在編譯時抓出不合理組合。

## 二、不可變、可變與遮蔽

```rust run
fn main() {
    let level = 1;
    let level = level + 1; // shadowing：建立新的同名綁定
    let mut hp = 100;
    hp -= 20;
    println!("level={level}, hp={hp}");
}
```

let 預設不可變：防止一個值在沒有心理準備時被修改，降低追蹤狀態的難度。let mut 才允許對同一變數指派。再次 let x 是遮蔽（shadowing），建立新的綁定，甚至可以改變型別；不是將原本的 immutable 偷偷改掉。

const MAX_PLAYERS: u32 = 3; 是編譯期常數宣告，必須指定型別，與一般變數不同。static 有固定存放位置，並帶有額外的共享與同步考量，不要混淆。

## 三、基礎型別從需求推導

整數：i8/i16/i32/i64/i128/isize 與無號 u8…usize。i 有正負、u 沒負值；位數影響可表示範圍。usize 常用於索引記憶體或集合長度，其大小跟目標架構有關。

浮點：f32、f64 常用於近似實數，不能保證十進位小數完全精確；金額系統應評估整數最小貨幣單位或十進位專用型別。

bool 是 true 或 false；char 是 Unicode scalar value，不等於一個「人眼看到的完整字」；例如某些表情符號由多個碼位組合。

tuple 把不同型別放一起，例如 (1, true)；array [i32; 3] 是固定長度、同一型別。長度可變集合 Vec<T> 留到第8章。

## 四、型別推導不是沒有型別

```rust fragment
let score = 10;           // 編譯器依上下文決定型別，通常是 i32
let ratio = 0.5_f64;      // 明確 f64
let coords: (i32, i32) = (3, 4);
let values = [10, 20, 30];
println!("{:?} {}", coords, values[0]);
```

Rust 是靜態型別；「省略型別標註」表示交由編譯器推導，並非執行到一半才決定。數值轉型一般需明確 as，但 as 並非總是安全無損；窄化可能截斷，安全轉換優先考慮 try_into / TryFrom。

## 五、函式為什麼區分語句與運算式？

```rust program
fn damage(base: i32, multiplier: i32) -> i32 {
    let result = base * multiplier;
    result
}
fn main() {
    let out = damage(12, 2);
    println!("{out}");
}
```

函式參數需標註型別；-> i32 是回傳型別。最後一行 result 沒有分號，是作為區塊回傳值的「運算式」。若寫 result; 就變成語句，值被丟棄，整個區塊結果為單位型別 ()，會與回傳 i32 的要求不符。

這個規則的根本：在 Rust 中 if、match、區塊都可以產生值。比喻：運算式是會交出成品的機器；語句是做完工作但不交成品的步驟。分號常讓一個運算式的結果被捨棄。

## 六、控制流程：分支與重複

```rust program
fn classify(hp: i32) -> &'static str {
    if hp <= 0 { "倒下" }
    else if hp < 30 { "危險" }
    else { "正常" }
}
fn main() {
    for n in 1..=3 { println!("第{n}回合"); }
    let mut stamina = 2;
    while stamina > 0 { stamina -= 1; }
    let twice = loop { break 2 * 5; };
    println!("{} {twice}", classify(15));
}
```

if 條件必須是 bool，不接受「0 當 false」的隱式轉換。if 作為運算式時，各分支型別需要能統一。for 常用於走訪迭代器；while 在條件為真時重複；loop 不設條件，使用 break 離開，甚至可 break 一個值。

## 七、真正底層需要警覺的錯誤

整數溢位：debug 組態通常在運行時 panic，release 行為會依編譯設定採用包覆等策略；不能以某個組態來賭業務正確性。需要時明確使用 checked_add、saturating_add 或 wrapping_add。

浮點 0.1 + 0.2 == 0.3 通常為 false，不是 Rust 壞掉，而是二進位有限精度表示使某些十進位小數無法精確存放。

索引越界會 panic；例如 arr[9] 不會自動擴展陣列。安全索引用 arr.get(index)，得到 Option。

## 八、練習／驗收

練習1：寫一個函式 clamp_hp(hp: i32) -> i32，將血量限制在 0 到 100。可用 hp.clamp(0,100) 或 if 運算式。

練習2：寫計算平均值，但不能假設整數相除會保留小數；比較 f64 轉換的結果及風險。

練習3：把一個 let x = 1 改成 let mut x = 1，指出語意差異；再測試 let x = "hello" 的遮蔽。

練習4：解釋為何 fn f() -> i32 { 3; } 編譯失敗。答案：最後運算式被分號丟棄，區塊結果是 ()。

驗收問題：何時用 const？為何 if 不把整數當布林值？char 為何不保證一個可見字形？為何不應使用 == 比較浮點近似值？

參考：<https://doc.rust-lang.org/stable/book/ch03-00-common-programming-concepts.html>
