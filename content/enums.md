# 第06章｜列舉與模式配對：先讓資料本身說清楚

> **一個任務怎麼可能同時開始又完成？** <!-- callout:context -->
>
> 任務只能是未開始、進行中或完成。如果用三個獨立開關，可能同時打開，變成不合理狀態。列舉（enum）讓同一個值在某一刻只能屬於定義好的其中一種變體。

## 一、問題：為何一堆 bool 容易出錯？

假設任務同時有 is_pending、is_running、is_done 三個布林值，可能三個都 true，產生互相矛盾的狀態。若真實世界只允許其中一種情況，資料型別應直接表達這個限制。

```rust fragment
enum QuestState {
    Pending,
    Running { progress: u8 },
    Done { reward: u32 },
}
```

enum 是「數種可能形態中擇一」，每一分支（variant）可攜帶不同資料。比喻：餐廳訂單只能處於待製作、製作中、已完成之一，每種狀態各有不同欄位；而不是每個狀態都用會彼此衝突的開關。

> **同一盞號誌，一次選一種狀態** <!-- callout:analogy -->
>
> 像交通號誌的狀態選單：選紅燈、黃燈或綠燈，而不是三個任意開關。每個 enum 變體還可以帶自己的資料，像任務進行中才有進度，完成時才有獎勵。

## 二、以 match 解構資料

```rust continuation
// 接續前一段 QuestState 定義。
fn status(q: QuestState) -> String {
    match q {
        QuestState::Pending => "尚未開始".to_string(),
        QuestState::Running { progress } => format!("進度 {progress}%"),
        QuestState::Done { reward } => format!("獲得 {reward} 金幣"),
    }
}
```

match 不是一般的 if 串接，而是依資料形狀取值；編譯器檢查分支涵蓋完整。若新增 Cancelled 變體而沒補 match，編譯就提醒你重新檢查業務邏輯。

> **任務的不同狀態可以帶不同資料** <!-- callout:walkthrough -->
>
> `enum QuestState` 定義允許的狀態；`Pending` 沒有附帶值，`Running { progress: u8 }` 帶進度，`Done { reward: u32 }` 帶獎勵。`match` 依目前變體執行對應分支，也能取出附帶的資料。

## 三、Option 為何比隨便用 null 安全？

當搜尋可能沒有結果，回傳值應表達「有一個值」或「沒有值」：

```rust program
fn find_bonus(level: u32) -> Option<u32> {
    if level >= 10 { Some(200) } else { None }
}
fn main() {
    match find_bonus(5) {
        Some(v) => println!("獎勵 {v}"),
        None => println!("尚未解鎖"),
    }
}
```

Option<T> 是標準庫 enum；沒有結果時是 None，而不是某個可能被誤用的魔法數字（-1、0）。取得內容前必須處理「不存在」這件事。unwrap() 在 None 會 panic，只適合你能證明一定存在的場合或快速測試。

## 四、Result 對應另一種問題

Option 關心有沒有；Result<T,E> 關心成功或失敗，以及失敗的原因。用 Result 不能取代所有 Option，也不是遇到錯誤就一定 panic。

```rust fragment
fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
    if b == 0 { Err("除數不可為 0") }
    else { a.checked_div(b).ok_or("除法溢位") }
}
```

這個範例順便示範邊界：i32::MIN / -1 也可能溢位，checked_div 會處理。

> **沒有值，不一定代表出錯** <!-- callout:warning -->
>
> `Option<T>` 表示有值 Some 或無值 None；`Result<T, E>` 表示成功 Ok 或失敗 Err。兩者表達的問題不同；None 不必然是錯誤，也不該把無值強行轉成某個數字。

## 五、if let、let else 與匹配適用範圍

```rust fragment
let maybe = Some(8);
if let Some(v) = maybe {
    println!("{v}");
}
```

當只在意單一模式，if let 比多分支 match 精簡；但不能因此忘記未匹配時怎麼辦。

```rust fragment
fn show(maybe: Option<u32>) {
    let Some(v) = maybe else {
        println!("沒有數字");
        return;
    };
    println!("{v}");
}
```

let else 要求不匹配的 else 分支離開當前流程（例如 return），所以之後可確定 v 存在。

## 六、模式中的借用與所有權

match 若按值處理含 String 的 enum，可能移動資料。使用 match &value 或 ref 相關模式可避免不必要的移動。第4章的 move / borrow 規則在 enum 不會消失。

## 七、實作與驗收

練習1：新增 QuestState::Failed { reason: String }，看編譯器指出哪個 match 尚未涵蓋。這就是型別系統協助你重構。

練習2：寫 fn first_even(xs: &[i32]) -> Option<i32>，找不到就 None；使用迴圈回傳 Some(*x)。

練習3：把三個 bool 改成 enum，說明排除了哪些不合理組合。

常見錯誤：None 直接拿去加法；Rust 要求明確拆封 Some。另一個錯誤是 match 漏掉分支；只有你確實不在意時才使用 _。

驗收：能說出 Option 和 Result 的差異、match 完整性帶來的維護效益，以及枚舉解構時所有權的方向。

> **讓門只能處於合法狀態** <!-- callout:practice -->
>
> 為遊戲設計 `Door` 的三種狀態：鎖住、關閉、開啟。用 match 分別顯示文字，再增加一種狀態，觀察原有 match 是否還完整。

參考：<https://doc.rust-lang.org/stable/book/ch06-00-enums.html>
