# 第19章｜模式：用資料形狀描述邏輯

## 一、模式為何比 if 更有表達力？

if 只判斷條件真假；模式能同時檢查資料屬於哪種形態並把其中欄位命名。Rust 讓同一套模式出現在 let、match、if let、while let、for 與函式參數中。

```rust fragment
let (x, y) = (10, 20);
println!("{x} {y}");
```

這裡 (x, y) 就是解構 tuple 的模式。模式不只是 match 專用。

## 二、可反駁（refutable）與不可反駁（irrefutable）

let (x,y) = (1,2) 必然成功，是不可反駁模式；Some(x) 可能在 None 上失敗，是可反駁模式。所以：

```rust fragment
let maybe = Some(7);
if let Some(n) = maybe { println!("{n}"); }
```

一般 let 不允許未處理的可反駁模式；let...else 則透過必須離開的 else 分支補上失敗情況。

## 三、結構體與列舉解構

```rust fragment
struct Point { x: i32, y: i32 }
let p = Point { x: 3, y: 8 };
match p {
    Point { x: 0, y } => println!("位於 Y 軸：{y}"),
    Point { x, y: 0 } => println!("位於 X 軸：{x}"),
    Point { x, y } => println!("其他：{x},{y}"),
}
```

模式可依欄位名稱取值，忽略不關心欄位；這比靠位置索引更容易閱讀，尤其在型別演進時。

## 四、_、..、範圍與多種候選

_ 忽略某一項；.. 在適用模式中省略剩餘元素；1..=5 匹配包含端點的範圍；A | B 代表多個候選模式。例如：

```rust fragment
let n = 3;
match n {
    1 | 2 => println!("低"),
    3..=5 => println!("中"),
    _ => println!("高"),
}
```

分支順序很重要：match 會採用第一個符合的分支；前面若寫 _，後面其他分支會變成不可到達。

## 五、match guard 的作用與限制

```rust fragment
let n = Some(5);
match n {
    Some(x) if x % 2 == 0 => println!("偶數 {x}"),
    Some(_) => println!("其他數字"),
    None => println!("沒有值"),
}
```

if 是額外守衛條件，並非模式本身。守衛會影響編譯器能證明的完整性，所以不要依賴它取代對其他型態的處理。

## 六、@ 綁定：同時檢查和保留值

```rust fragment
let age = 18;
match age {
    n @ 18..=30 => println!("範圍內：{n}"),
    _ => println!("範圍外"),
}
```

@ 左側替整個匹配值取名字；這在複雜 enum 解構與範圍匹配中特別實用。

## 七、模式與所有權的交叉

```rust fragment
let name = Some(String::from("Ada"));
match &name {
    Some(s) => println!("{s}"),
    None => {}
}
```

match &name 表示匹配參考，常能借用內部資料而不搬走 String。若直接 match name 並綁定 String，可能消耗其中值。模式匹配也受 borrow checker 規則支配。

## 八、錯誤情境

過度使用 _ 會掩蓋 enum 新增分支後應重新考慮的業務邏輯；對容易擴充的 enum，要權衡前向相容需求與完整性檢查。

把 guard 裡做複雜副作用會使控制流程難追蹤；守衛適合補充條件，不宜變成另一套隱藏狀態機。

## 九、練習與驗收

題1：解構 (u32, Option<&str>) 並分別處理有字串與無字串。

題2：使用 if let 和 match 寫一樣的邏輯，比較可讀性與完整性。

題3：在 QuestState::Running { progress } 寫守衛：大於等於 100 提醒完成，其他回報進度。

題4：對 String 值分別測 match value 與 match &value，預測後續可否再使用。

驗收：能判斷模式是否可反駁、識別解構移動或借用，以及 match 為什麼需要窮盡處理。

參考：<https://doc.rust-lang.org/stable/book/ch19-00-patterns.html>
