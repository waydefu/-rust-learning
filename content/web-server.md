# 第21章｜從位元組串流理解 HTTP 伺服器

> **來龍去脈**
>
> 在瀏覽器輸入網址後，瀏覽器會向另一個程式提出請求，伺服器再把回應傳回來。最後一章將前面學過的字串、錯誤、所有權、執行緒和資源管理放進一條完整的資料流。

## 一、為什麼最後要做 Web Server？

這個專案把 I/O、錯誤處理、所有權、執行緒、資源管理放在同一流程：程式必須等待連線、讀取位元組、解析請求、產生回應。它也會讓你理解生產級 HTTP 框架替你處理了多少細節。

注意：以下只是本機學習原型，不應直接暴露公網；不具備完整 HTTP 解析、安全防護、TLS、持續連線與高負載防護。

> **生活比喻**
>
> TCP 像可靠、有順序的位元組傳輸管道，HTTP 像雙方約定的請求與回應格式。伺服器監聽本機地址與連接埠，接受連線，讀取請求，再按格式寫回回應；TCP 不保證一次 read 就收到完整 HTTP 訊息。

## 二、TCP、HTTP 的分工

TCP 是有序可靠的位元組串流，不知道「一個請求」從哪裡到哪裡；HTTP/1.1 在其上定義請求行、標頭、內容和回應格式。

比喻：TCP 像確保信件內文字依序送達的運輸管道；HTTP 像雙方約定的信件格式與欄位。比喻的邊界：TCP recv 可能只收到部分資料，也可能收進多段資料，讀一次不代表得到完整 HTTP request。

> **程式拆解**
>
> `TcpListener` 負責等連線，`TcpStream` 代表已連線的雙向資料流；`read` 讀入最多指定數量的位元組並回傳實際長度 n；`&buffer[..n]` 只查看這次收到的資料。回應中的 Content-Length 必須按位元組數計算。

## 三、建立最小本地伺服器

在新 Cargo binary 專案的 src/main.rs 放入：

```rust program
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0_u8; 1024];
    let n = stream.read(&mut buffer)?;
    let is_root = buffer[..n].starts_with(b"GET / HTTP/1.1\r\n");
    let (status, body) = if is_root {
        ("200 OK", "Hello from Rust")
    } else {
        ("404 Not Found", "Not Found")
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle(stream) { eprintln!("{e}"); }
            }
            Err(e) => eprintln!("連線錯誤：{e}"),
        }
    }
    Ok(())
}
```

先 cargo run，再從同一電腦的瀏覽器開啟 <http://127.0.0.1:7878/>。請不要期待這個簡易讀取器能正確處理所有合法 HTTP 請求。

## 四、逐層拆解到底發生了什麼

TcpListener::bind 綁定本機網路位址與連接埠；incoming 依序等候連線；TcpStream 代表與某個客戶端的連線；read 取得部分位元組；starts_with 僅做非常簡化的判斷；write_all 盡力把整份回應寫入連線。

Content-Length 應以位元組數表示，所以 body.len() 適用於 UTF-8 String 的位元組長度；不能用 chars().count() 冒充，否則非 ASCII 回應可能出錯。

> **常見陷阱**
>
> 教學原型可能只讀一次、只辨認最簡單的請求，還沒有完整協定解析、逾時與資源限制。就算加入執行緒池，也不代表能安全對外服務。先限定在本機回環位址測試，不將本例直接暴露公網。

## 五、這份示範的關鍵缺陷：阻塞

目前每個連線都在主執行緒順序處理。若一個客戶端連線後不送資料，read 可能長時間等待，後面的連線就排隊。這就是為什麼官方書接著要介紹 ThreadPool。

## 六、如何從每連線一執行緒演進到執行緒池？

最直覺的改造：接收後 thread::spawn(move || handle(stream))，可讓多個連線交錯處理。但連線量大時無限制建立執行緒很昂貴。

執行緒池預先建立有限個 worker，透過通道傳遞待執行工作；各 worker 從佇列取任務。核心是有界資源、排隊規則、工作失敗隔離與關閉流程。

概念設計：TcpListener → 有界佇列 → N 個 worker → handle(TcpStream) → 回應。

實作的核心難點不是寫一行 spawn，而是：佇列滿了怎麼辦？某個工作 panic 怎麼辦？如何停止接新連線並等待現有任務？如何保證共享狀態安全？

### 實作練習：有界工作佇列和優雅關閉

先用簡單工作驗證排程。改造教材伺服器時，先為 socket 設定讀寫逾時，再用 `move` 閉包取得 `TcpStream` 的所有權。閉包內呼叫 `handle(stream)` 並處理回傳的 `Result`，最後把閉包送入工作佇列。

```rust run
use std::sync::{Arc, Mutex, mpsc::sync_channel};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

fn main() {
    let (sender, receiver) = sync_channel::<Job>(4);
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::new();

    for _ in 0..2 {
        let receiver = Arc::clone(&receiver);
        workers.push(thread::spawn(move || {
            loop {
                let next = {
                    let guard = receiver.lock().expect("接收端的鎖已中毒");
                    guard.recv()
                };
                match next {
                    Ok(job) => job(),
                    Err(_) => break,
                }
            }
        }));
    }

    let (result_sender, results) = std::sync::mpsc::channel();
    for id in 0..4 {
        let reporter = result_sender.clone();
        sender.send(Box::new(move || reporter.send(id).unwrap())).unwrap();
    }
    drop(sender);
    drop(result_sender);

    for worker in workers {
        worker.join().expect("工作執行緒發生 panic");
    }
    let mut done: Vec<_> = results.into_iter().collect();
    done.sort_unstable();
    assert_eq!(done, [0, 1, 2, 3]);
}
```

鎖只保護 `recv()`，取到工作後立即離開區塊。若執行工作時仍持有鎖，池裡的兩個 worker 就無法同時執行任務。排序讓測試只驗證四個工作都完成，不假設完成順序。最後一個發送端被丟棄後，接收端仍會取完佇列，之後才收到斷線並退出。

本例的 `send` 會在佇列滿時等待。若應用要在忙碌時快速拒絕，可用 `try_send`，明確處理 `Full` 與 `Disconnected`，並讓被拒絕的連線釋放。限制佇列容量可控制待處理工作數，單一工作若長期阻塞仍會占住 worker。

## 七、為何不能直接把這個原型拿去部署？

它沒有完整解析 HTTP 標頭、請求大小限制、超時、keep-alive、chunked body、TLS、路由、限速、對抗慢速連線與輸入驗證。特別是對陌生網路流量，單次 read + starts_with 沒有完整語意保障。

生產級應使用經維護的 HTTP server library / framework，搭配 logging、metrics、timeout、身份驗證、資源限制及安全更新。

## 八、實作階梯

第一關：本機回傳 200，路徑不匹配回傳 404。

第二關：將 handle 拆成解析、路由、回應三函式，為純邏輯寫測試。

第三關：改成 worker queue，設最大任務數與容量。

第四關：設計優雅關機，停止接收、通知 worker、等待已接任務完成。

第五關：加入 timeout、紀錄與壓力測試，將其限制與成熟框架比較。

> **動手驗收**
>
> 在本機依序測試根路徑和不存在的路徑，預測 200 與 404。接著畫出一個請求從連線、排隊、工作執行緒到回應的流程；指出如果有十個工作但只有四個 worker，其餘工作應等待或被拒絕的策略。

## 九、驗收

能說清楚 TCP 與 HTTP 不同、為何 read 不保證完整請求、Content-Length 為何看 byte、單執行緒如何阻塞、thread pool 如何控制資源、何時該改用 async server。

參考：<https://doc.rust-lang.org/stable/book/ch21-00-final-project-a-web-server.html>
