# 第17章｜非同步不是讓電腦突然多一顆核心

新版官方教材為非同步設立了獨立章節。非同步語法從 Rust 2018 Edition 起可用，本章以 Rust 2024 Edition 示範。

> **等待網路時，CPU 可以做別的事** <!-- callout:context -->
>
> 點餐後等待料理時，你可以先去做別的事，而不是一直站在廚房門口。網路和磁碟也經常需要等待；非同步讓程式在等待期間把執行機會交給其他工作，而不是每項等待都占著一條專用執行緒。

## 一、為何有執行緒還需要非同步？

當程式等待網路或磁碟回應時，CPU 可能暫時沒有實際計算工作。若每個等待都占用專屬 OS 執行緒，數量很大時執行緒記憶體和排程成本會明顯增加。非同步將許多可能等待的操作交給執行時系統協調，在可暫停點切換其他工作。

比喻：餐廳只有一位服務生，送餐後不必站在桌邊等客人吃完，能先服務下一桌；await 就是允許暫停的地方。它不代表兩個人同時搬盤子，CPU 密集工作也不會因為加 async 而自動平行化。

## 二、async fn 回傳的不是當下結果

```rust program
async fn answer() -> u32 {
    42
}
fn main() {
    let future = answer();
    let _ = future;
}
```

呼叫 answer() 產生 Future，而不是立刻執行完取得 u32。Future 是描述未來可產生結果的狀態機抽象，通常需由執行器（executor）反覆 poll 才有進展。上面 main 不會印出 42，也沒有真正等待 future 結果。

> **呼叫 async fn 不等於開始執行** <!-- callout:walkthrough -->
>
> `async fn answer() -> u32` 描述最終產出 u32 的非同步函式，但 `answer()` 當下建立的是 Future。只有被執行器 poll，函式本體才會推進。`.await` 等待結果，若目前還未完成，可以暫停當前 Future，讓執行器安排其他工作。

## 三、.await 做了什麼？

```rust continuation
// 接續前一段 answer 定義。
async fn double() -> u32 {
    let n = answer().await;
    n * 2
}
```

執行器呼叫外層 Future 的 `poll` 時，`.await` 會輪詢內層 Future。內層若回傳 `Ready(value)`，外層可直接使用結果。若回傳 `Pending`，外層保存下一次繼續所需的狀態，並把等待交回執行器。當計時器或 I/O 通知可再試時，由已註冊的 `Waker` 通知執行器重新排程。喚醒不保證下一次就能完成。

跨越暫停點仍要使用的區域值會留在 Future 的狀態中。因此工作若由 Tokio 的多執行緒 `spawn` 排程，活過 `.await` 的資料也可能影響整個 Future 是否符合 `Send`。例如先取得 `std::sync::MutexGuard`，在 `.await` 前用區塊結束守衛作用域，通常比把守衛跨暫停點保存更清楚。

比喻邊界：await 不保證會暫停，也不是等待時自動創一個執行緒；若所等待的 future 當下就準備好，可能直接繼續。

> **取餐單如何對應 Future** <!-- callout:analogy -->
>
> Future 像一張尚未完成的取餐單；executor 是安排何時檢查工作進度的執行器；poll 是詢問進度，Pending 表示還沒好，Ready 表示結果已準備好，Waker 用來通知執行器之後應重新排程。不是每個 Future 都必須等待，第一次 poll 也可能完成。

## 四、Rust 標準庫為何不能單獨「執行」所有 async 工作？

標準庫提供 Future、Poll、Pin 等底層抽象，但實際 I/O 驅動、工作排程與計時常由 Tokio、async-std 等外部 runtime 提供。對一般新手專案可先選 Tokio 學習，不要立刻自行實作 executor。

示例（建立專案後使用 cargo add tokio@1.54.1 --features macros,rt-multi-thread,time）：

```rust project external:src/bin/timer.rs
#[tokio::main]
async fn main() {
    let a = tokio::time::sleep(std::time::Duration::from_millis(100));
    a.await;
    println!("等待結束");
}
```

請先在 Cargo.toml 確認 tokio 相關 features 已啟用。此程式只是示範非同步等待，不是平行運算測試。

## 五、並行與非同步的不同調度模型

執行緒由 OS 調度，可能在多核心同時運算；async task 通常由 executor 在碰到可讓出的位置協調。runtime 可以使用多個 OS 執行緒，因此 async 和 threads 不是互斥選項。

CPU-bound（大量計算）可能需要 thread pool 或專門的阻塞任務 API；I/O-bound（網路、檔案等待）常受益於非同步。

> **async 不會讓阻塞操作自動變快** <!-- callout:warning -->
>
> async 不會自動增加 CPU 核心，也不是任意阻塞函式的加速器。在 async 任務裡呼叫阻塞操作仍可能卡住執行緒；長時間 CPU 運算也需要合適的工作分配。取消 Future 時要考慮已完成的外部副作用。

## 六、join、spawn 與等待多項工作

只寫 a.await; b.await; 通常是先完成 a，再處理 b；將 futures 以 join! 等方式共同驅動，才可使它們在等待期間交錯進展。

spawn 則將 future 交給 runtime 排程；可能涉及 Send、'static 等要求。注意 spawn 不等於保證與當前程式同時執行，更不保證固定順序。

## 七、Pin / Unpin 為何是難點？

某些 future 狀態機可能在暫停期間持有與自身儲存位置相關的參考。若任意移動底層記憶體可能破壞有效性。Pin<P> 是一種用型別約束「特定資料不可隨意被移動」的工具，正確理解需區分移動指標與移動被指向資料，也需遵守 pinning 契約。

新手不必先寫 unsafe Pin 程式，但要知道 Pin 不等於「資料永遠放在 heap」，也不是自動記憶體鎖。

## 八、Stream 與 Future 的差異

Future 通常表示一個最終結果；Stream 表示一連串可陸續產生的項目。就像一次下載完成通知對比持續的聊天訊息更新。但 Rust 的 Stream 使用需依 crate 與 runtime 的實作 API，並非只寫 for 就能直接處理所有非同步資料流。

## 九、取消、安全與錯誤

Task 被取消或 future 被 drop，尚未完成的操作可能因此停止；如果已經對外部系統造成部分副作用，取消不會自動復原。重要流程要想「中途取消會留下什麼狀態」、「如何避免重複扣款或重複寫入」。

不要在 async task 內長時間持有同步 Mutex guard 跨 await，可能妨礙排程、導致死鎖或不符合 Send 要求。必要時縮短鎖範圍或改用適當的 async mutex。

## 十、驗收

題1：為何呼叫 async fn 卻沒有 .await 可能不執行主要邏輯？

題2：一個 async 函式內使用 std::thread::sleep 可能有什麼問題？它會阻塞執行緒，而不是合作式等待。

題3：比較「十個網路請求」與「十個大量矩陣運算」各適合的排程策略。

題4：何時用 join!、何時用 spawn，取消會影響哪些外部副作用？

驗收：能清楚區分 async、Future、await、executor、task、OS thread、parallelism；知道非同步不是免費加速。

> **比較建立 Future 與真正 await** <!-- callout:practice -->
>
> 先建立一個 async 函式，只呼叫、不 await，猜測函式本體是否會印字。再使用本章的 Tokio 執行範例 await 它，觀察差異。最後解釋兩個等待工作為何可以交錯推進，卻不一定平行執行。

參考：<https://doc.rust-lang.org/stable/book/ch17-00-async-await.html>
