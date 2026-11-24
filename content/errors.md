# 第09章｜錯誤不是例外情況，而是程式設計的一部分

> **打不開存檔，遊戲該怎麼辦？** <!-- callout:context -->
>
> 假設你要打開一個存檔，可能檔案不存在、格式壞掉，也可能讀取成功。若把任何失敗都當成程式完全不能運作，使用者很難補救。Rust 希望你在寫程式時就想好：失敗時應重試、回報，還是停止目前流程？

## 一、先把「出錯」分兩類

程式的錯誤有不同意義：一種是內部假設已被破壞、程式無法合理繼續（例如不可能的狀態）；另一種是外界正常可能發生的失敗（檔案不存在、權限不足、輸入無效）。Rust 用 panic! 與 Result 分別表達常見處理方向。

比喻：餐廳沒收到食材，是預期可發生的營運異常，需要回報與補救；廚房的安全規範完全失效，則可能必須立即停止作業。不能把所有失敗一律當成當機，也不能永遠吞掉錯誤。

> **空盒子、送貨失敗與停止作業** <!-- callout:analogy -->
>
> `Option` 像問「盒子裡有東西嗎」；`Result` 像問「送貨成功嗎，失敗原因是什麼」；`panic!` 表示目前流程遇到無法按正常方式繼續的情況。panic 不一定等於整個程序立刻終止，還要看 unwind 或 abort 設定。

## 二、panic! 的行為與用途

```rust program
fn main() {
    let x = [1, 2, 3];
    let index = std::env::args().count();
    println!("{}", x[index]);
}
```

執行時傳入三個額外參數（程式名稱也算一個）時，`index` 為 4，讀取三元素陣列會 panic。資料在執行時取得，才能觀察越界檢查。若直接寫死 `x[10]`，編譯器可能先因 `unconditional_panic` 拒絕建構。

panic 可進行 stack unwinding，也可能依編譯策略直接 abort；不是一般的 Result 回傳。它適合表示程式無法維持必要不變條件的情況，但不宜用於使用者可能輸入不合法數字這類正常事件。

> **從陣列索引看見 panic** <!-- callout:walkthrough -->
>
> 第一段程式用 `std::env::args().count()` 取得命令列參數數量，再以它當陣列索引；一般含程式名稱的參數數量至少為 1。當索引超過合法的 0 到 2，`x[index]` 會 panic。改用 `get` 就能把是否有值表示成 Option。

## 三、Result 的型別結構

```rust fragment
enum Result<T, E> { Ok(T), Err(E) }
```

這是簡化展示的觀念，標準庫已有此型別，不要自行定義同名版本來替換。它同時記錄成功時的資料 T 與失敗時的錯誤 E。呼叫者必須選擇：修復、轉換、往上回傳或明確停止。

```rust program
use std::fs::File;
fn main() {
    let file = File::open("notes.txt");
    match file {
        Ok(_) => println!("開啟成功"),
        Err(e) => eprintln!("無法開啟：{e}"),
    }
}
```

## 四、為何要有 ?：避免每層重複樣板碼

```rust fragment
use std::fs;
use std::io;
fn read_config() -> Result<String, io::Error> {
    let text = fs::read_to_string("config.txt")?;
    Ok(text.trim().to_string())
}
```

? 在 Ok(value) 時取出值並繼續；在 Err(error) 時從目前函式提前回傳相容的錯誤（必要時經 From 轉換）。它不是「忽略錯誤」，反而是讓失敗沿著呼叫鏈以型別安全的方式往上傳。

函式通常必須回傳相容的 Result 或其他支援 Try 語意的型別；不能任意放在普通回傳 i32 的函式裡。

> **問號會把錯誤交給呼叫者** <!-- callout:warning -->
>
> `?` 不是忽略錯誤：在 Result 回傳的函式中，遇到 Err 會提早回傳（必要時轉換錯誤型別）；成功才取出內部值繼續。`unwrap()` 則可能 panic，因此不應用它逃避常見的外部輸入失敗。

## 五、應該在哪個層級處理？

底層函式盡量附上具體上下文並回傳 Result；應用邊界（例如 main 或 HTTP handler）決定要顯示訊息、重試、回應狀態碼或退出。處理太早會失去上層決策權，處理太晚則可能缺乏診斷上下文。

## 六、何時用 Option、Result、panic？

查詢可能「沒有」但不是失敗：Option。操作可能失敗且想知道原因：Result。不變條件被破壞、無法合理繼續：panic。不要把「沒找到使用者」和「資料庫壞了」都用 None 混在一起。

## 七、自訂錯誤型別從需求推導

應用可能出現多個錯誤來源：讀檔、解析、驗證。可用 enum AppError { Io(io::Error), InvalidInput(String) } 表達，讓上層能匹配並處理。需要易讀的錯誤訊息時可實作 Display 與 Error，或使用經評估的 thiserror 等外部 crate。

```rust fragment
fn parse_level(text: &str) -> Result<u32, String> {
    let n = text.parse::<u32>().map_err(|e| format!("數字無效：{e}"))?;
    if n > 100 { Err("超過上限".into()) } else { Ok(n) }
}
```

map_err 把底層錯誤轉換為本函式想公開的錯誤型別。錯誤型別的選擇是 API 合約，不只是為了讓程式通過編譯。

## 八、常見反模式

大量 unwrap()：只要外部輸入一變就 panic；用 unwrap_or_default()：可能悄悄掩蓋真的錯誤；完全忽略 Result：呼叫者以為操作成功。

適當使用 expect("具體假設") 可標記「此狀況按設計不可能」；例如已在上游嚴格驗證的內部值，但這不是萬靈丹。

## 九、練習與答案

實作 fn load_score(path: &str) -> Result<u32, Box<dyn std::error::Error>>：read_to_string(path)?，trim().parse::<u32>()?，最後 Ok(score)。Box<dyn Error> 可讓不同錯誤以共同介面回傳，簡化應用邊界，但失去部分具體型別資訊。

練習：分別輸入不存在路徑、非數字檔案、正常數字；驗證三種不同結果。

驗收：能解釋 ? 不是 try/catch、何時使用 Err 與 panic、如何保留錯誤資訊、如何判斷資料不存在和系統故障。

> **試著處理四種數字輸入** <!-- callout:practice -->
>
> 寫一個將使用者文字轉為整數的函式，測試 42、abc、空字串與超大數字。對失敗回傳 Result，讓呼叫者決定提示重試或結束，說明為何不能全部回傳 0。

參考：<https://doc.rust-lang.org/stable/book/ch09-00-error-handling.html>
