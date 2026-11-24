# 第02章｜用猜數字遊戲理解程式的生命週期

本章不是教你照抄遊戲，而是理解程式如何接收外界資訊、轉換資料、判斷情況並重複執行。

版本：範例以 Rust 2024 Edition 為基礎。原書繁中第2章對照：<https://rust-lang.tw/book-tw/ch02-00-guessing-game-tutorial.html>

> **讓電腦從自言自語變成對話** <!-- callout:context -->
>
> 第一章的程式只會說 Hello。現在想讓它問你一個問題、聽取回答，再依答案作出不同反應。猜數字遊戲恰好把這三件事串在一起。我們先把答案固定為 42，等輸入與判斷都能正常運作，再讓電腦隨機出題。

## 一、根本問題：程式要如何「互動」？

第一天的 Hello World 只有單向輸出。真實應用必須讓外界提供資料。這帶出五個問題：從哪裡讀？資料最初是什麼型別？能否成功轉換？怎樣比較？如何讓錯誤不至於使程式失控？

比喻：收件員收到紙條「42」，先把紙條讀成文字，再確認它確實是數字，最後才交給裁判比較。不能因為紙條長得像數字，就假設它已經是可運算的整數。

> **為什麼 42 一開始是文字？** <!-- callout:analogy -->
>
> 人打進終端機的是文字，哪怕輸入 42，程式也先收到字串。`trim()` 去除首尾空白，`parse()` 嘗試把文字轉成數字；可能成功，也可能失敗。`match` 像分流的岔路，根據結果選擇下一步。

## 二、先以固定答案建立最小可用版本

請在 cargo new guess_game 建立的 src/main.rs 中輸入：

```rust program
use std::cmp::Ordering;
use std::io;

fn main() {
    let secret: u32 = 42;
    loop {
        println!("請輸入 1 到 100 的整數：");
        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input).expect("無法讀取輸入");
        if bytes_read == 0 {
            break;
        }
        let guess: u32 = match input.trim().parse() {
            Ok(n) => n,
            Err(_) => {
                println!("這不是有效的整數，請重試");
                continue;
            }
        };
        match guess.cmp(&secret) {
            Ordering::Less => println!("太小"),
            Ordering::Greater => println!("太大"),
            Ordering::Equal => {
                println!("猜對了！");
                break;
            }
        }
    }
}
```

執行 cargo run。第一輪故意輸入「abc」、空白、101、42，觀察每一步。

> **先讀進一行，再決定怎麼處理** <!-- callout:walkthrough -->
>
> 先看主程式：`let secret: u32 = 42;` 設定秘密數字，`loop` 重複，`String::new()` 準備空文字，`read_line` 讀取使用者輸入。`&mut input` 允許讀取函式修改 input，第四章會解釋借用的完整原理。

## 三、從語法拆解因果

use 的作用是將名稱帶入當前作用域，並沒有複製函式庫。std 是 Rust 標準函式庫；io 是輸入輸出模組；Ordering 是描述比較結果的列舉。

let 建立變數綁定；let mut 允許對綁定變更或可變借用。String::new() 建立可擴充的 UTF-8 字串。輸入不是直接存進 u32，因為人可能輸入文字、空白或超範圍數字。

&mut input 意思是把 input 的可變借用交給 read_line。函式可修改原本的字串，而非只拿走一份拷貝。借用的嚴格規則由第4章推導。

read_line 的回傳型別是 Result<usize, io::Error>；Ok 包含讀取的位元組數，Err 包含錯誤。回傳 `Ok(0)` 表示輸入已到末尾，跳出迴圈可避免管線結束後重複處理空字串。expect 遇到 Err 時會終止程式並顯示訊息；正式系統通常需要更好的錯誤傳播。

trim() 去除兩端空白與換行。parse() 嘗試將文字轉成指定型別，透過左側的 : u32 決定目標。失敗時傳回 Err 而不是偷偷替你猜個數字。

match 是「根據資料的實際形狀選擇分支」。在 parse 結果為 Ok(n) 時取出 n；Err(_) 代表錯誤不論內容，_ 是忽略值。continue 回到下一輪，break 跳出 loop。

cmp 回傳 Ordering::Less、Equal 或 Greater；這不是三個任意字串，而是型別系統知道的三種互斥情況，編譯器能檢查 match 是否涵蓋所有分支。

## 四、為何要分兩次 match？

第一次處理「資料是否合法」，第二次處理「合法資料的業務規則」。這是分離關注點：輸入錯誤不應被誤判成猜太大；猜錯也不是 I/O 故障。

反例：直接 input.parse::<u32>().unwrap()，對「abc」會 panic，程式中止。初學時可以用來觀察錯誤，但不適合允許使用者重試的場景。

> **0 和 101 也是合法的 u32** <!-- callout:warning -->
>
> 輸入 abc 不應被當成 0，也不應讓遊戲直接崩潰。空行、非數字、讀到 EOF 是不同情況；還要留意 `u32` 接受 0 和大於 100 的數字，因此若規則限制 1 到 100，需要另外檢查範圍。

## 五、加入隨機數：理解依賴而非背 API

執行 `cargo add rand@0.9.5`，查看 Cargo.toml 新增的版本依賴。rand 是第三方 crate，不是 Rust 語言內建語法。rand 0.9 系列的簡便用法：

```rust project external:src/bin/random.rs
let secret: u32 = rand::random_range(1..=100);
```

把原本的 42 換成這行。1..=100 表示包括 100；1..100 不包括 100。若版本相容性不同，應依 Cargo.lock 與 <https://docs.rs/rand/> 確認 API；隨機數並不代表密碼學安全。

## 六、抽象與底層如何銜接？

一個程式在記憶體中同時持有 String 的緩衝區、整數、錯誤狀態等。read_line 可能向作業系統要求讀取標準輸入；Cargo 只在編譯建構時工作，執行時不需要 Cargo 持續主持。

比喻的邊界：「輸入紙條」是直覺模型；實際上 stdin 讀的是位元組串流，文字編碼、緩衝、換行、平台差異都會影響行為。

## 七、練習與答案線索

練習1：超出 1–100 的合法整數要另外提示。線索：在 cmp 前加 if !(1..=100).contains(&guess) { ... continue; }。

練習2：統計猜測次數。線索：let mut attempts = 0;，在成功 parse 後 attempts += 1。

練習3：如果輸入負數會怎樣？由於 u32 不表示負數，parse 會回 Err；若要接受負數，改用 i32 再驗證範圍。

練習4：為什麼 trim 不等於 parse？前者處理字串外觀，後者判斷能否表示為指定數值。

驗收：能獨立解釋 String→trim→parse→Result→match→Ordering 的每一步；故意輸入錯誤時知道哪個分支接住；能不用看稿改出計次功能。

> **自己當一次遊戲裁判** <!-- callout:practice -->
>
> 固定答案 42，依次輸入 20、abc、80、42，預測每次應走哪個分支。再加入猜測次數，自己決定無效輸入是否計數，並解釋這個規則。

## 延伸來源

Rust 官方最新版第2章：<https://doc.rust-lang.org/stable/book/ch02-00-guessing-game-tutorial.html>

rand API：<https://docs.rs/rand/latest/rand/fn.random_range.html>
