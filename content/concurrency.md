# 第16章｜無懼並行不是無視競爭

## 一、先區分三個概念

並行（concurrency）是多項工作在重疊時間內取得進展。平行（parallelism）是多項工作在同一時刻執行。資料競爭（data race）是對同一記憶體位置的衝突存取，至少有一方為寫入，且缺少必要的同步。即使使用原子操作消除了資料競爭，兩個操作的相對順序仍可能造成邏輯競態。

比喻：一位廚師交替煮兩道菜是並行；兩位廚師同時煮是平行；兩位廚師不協調地修改同一張配方卡可能導致混亂。比喻的邊界：真正資料競爭有特定記憶體模型定義，不等同所有可能產生錯誤順序的邏輯競爭。

## 二、執行緒與所有權

```rust run
use std::thread;
fn main() {
    let name = String::from("worker");
    let handle = thread::spawn(move || {
        println!("來自 {name}");
    });
    handle.join().expect("執行緒失敗");
}
```

spawn 建立 OS 執行緒；move 閉包取得 name 所有權，避免新執行緒持有可能已被呼叫者釋放的區域資料。join 等待執行緒結束並回報 panic 等結果；沒有 join 不代表必然等到它工作完成。

## 三、訊息傳遞：把共享狀態變成資料流

```rust run
use std::sync::mpsc;
use std::thread;
fn main() {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        tx.send(String::from("完成")).unwrap();
    });
    let message = rx.recv().unwrap();
    println!("{message}");
}
```

通道把資訊從傳送端交給接收端，send 消耗傳入的 String，所有權移給接收者。當傳送端消失、接收端消失等情況發生時，通道操作可能回傳錯誤；不能假設 recv 永遠成功。

## 四、共享狀態：為何不能只用 Rc<RefCell<T>>？

Rc 參考計數本身不提供跨執行緒同步；RefCell 的借用狀態也非跨執行緒安全。需要多執行緒共享時，常搭配 Arc<T>（原子參考計數）與 Mutex<T>（互斥鎖）。

```rust run
use std::sync::{Arc, Mutex};
use std::thread;
fn main() {
    let counter = Arc::new(Mutex::new(0_u32));
    let mut handles = Vec::new();
    for _ in 0..4 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut number = c.lock().unwrap();
            *number += 1;
        }));
    }
    for h in handles { h.join().unwrap(); }
    println!("{}", *counter.lock().unwrap());
}
```

鎖的核心：在 guard 存在時，其他需要同一把鎖的工作不能同時進入臨界區。RAII guard 離開作用域自動釋放鎖。鎖定、等待與上下文切換都有成本；不是所有共享值都該包 Mutex。

## 五、Mutex 不是萬無一失

互斥能防止特定同步問題，卻無法自動消除死鎖、鎖順序錯誤、效能瓶頸或業務層級競態。鎖中 panic 可能使 Mutex 處於 poisoned 狀態，lock() 回傳錯誤；真實系統需判斷資料是否仍可恢復。

RwLock 適用某些讀多寫少情境，Atomic 類型用於特定無鎖或細粒度同步，但需要理解記憶體順序，不應盲目用 Relaxed 代替鎖。

## 六、Send 與 Sync 到底是什麼？

Send：型別的值可以安全地跨執行緒轉移所有權。Sync：&T 可以安全地在執行緒間共享。它們是標記 trait，編譯器與標準庫使用它們限制跨執行緒行為。

不要把 Send 當「沒有任何危險」：跨執行緒的邏輯仍可能錯；它的保證集中在適當的型別與記憶體安全契約。

## 七、並行設計的選擇順序

能否避免共享？→ 能否把資料以通道轉交？→ 若必要共享，是否需 Arc + Mutex？→ 是否適用 RwLock / Atomic？→ 用實際效能測量驗證。細粒度鎖不一定更快，並行度更高也不一定吞吐量更高。

## 八、實作／驗收

題1：將四個工作者各算一組數字總和，透過 channel 傳回 main 合計。

題2：在 Mutex 範例中故意長時間保持 lock guard，觀察其他執行緒等待。

題3：說明 data race 與一般「結果先後順序不確定」的差別。

題4：為何不能直接跨執行緒移動 Rc<String>？

驗收：能畫出每個執行緒擁有何資料、共享了什麼、同步點在哪、哪裡可能形成死鎖。

參考：<https://doc.rust-lang.org/stable/book/ch16-00-concurrency.html>
