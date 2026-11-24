# Day 01｜開始入門：Rust、編譯與 Cargo

*對照《Rust 程式設計語言》繁中版 1.1 安裝教學、1.2 Hello, World!、1.3 Hello, Cargo!*

本日定位：真正理解「一段文字如何變成可執行的程式」，而非背下指令。建議邊閱讀邊自己輸入範例。Rust 2024 Edition 需要 Rust 1.85 或更新版本。本課的自動範例驗證固定使用 1.85.1，終端機可使用更新的穩定工具鏈。

> **按下跳躍鍵，電腦怎麼知道要跳？** <!-- callout:context -->
>
> 玩遊戲時，你按下跳躍鍵，角色就跳了起來。但 CPU 不懂「跳躍」這個詞。它只會執行機器指令。要把我們的想法變成這些指令，第一步就是用一種程式語言把規則寫下來。Rust 是其中一種；今天我們就寫一個最小的程式，看看這段文字如何變成螢幕上的結果。

## 一、今天學會什麼

學完後，你應能說清楚：原始碼、編譯器、執行檔、終端機各做什麼；讀懂第一個 Rust 程式的每個符號；用 rustc 與 Cargo 各自完成一次編譯執行；遇到基礎錯誤時知道往哪裡查。

重點不是現在就學會寫遊戲，而是建立後續每一章都會依賴的完整心智模型。

## 二、先從根本理解：電腦到底如何執行程式？

### 1. 為什麼不能直接把中文指令交給 CPU？

人可以理解「把一句話顯示到螢幕上」，CPU 卻不能直接理解自然語言或 Rust 原始碼。CPU 執行的是特定處理器架構的機器指令；因此，程式必須經過一個翻譯與建構過程。

比喻：你是設計師，Rust 原始碼是一張有規範的工程圖；編譯器像製造部門，負責檢查設計並產生可工作的零件與組合成果；作業系統像工廠管理員，負責把成品載入、配置資源並啟動；CPU 則按已形成的指令工作。比喻只幫你理解分工，真實編譯還涉及型別檢查、最佳化、連結等步驟。

從程式到結果，流程是：

想法 → Rust 原始碼（.rs） → rustc 編譯與連結 → 可執行檔 → 作業系統啟動 → CPU 執行 → 終端機看到輸出

請留意「編譯」與「執行」是兩件事。編譯一次得到執行檔，之後通常可以多次執行它，而不必每次重新編譯（前提是該執行檔與目標系統相容）。

### 2. Rust 為什麼特別強調安全？

系統程式常直接管理記憶體，錯誤可能造成讀取無效位置、重複釋放等問題。Rust 想保留接近底層的效能與控制能力，同時把許多危險情況提前到編譯時擋下來。第 4 章會完整理解所有權與借用；今天先知道設計動機即可。

比喻：有些語言像讓工人先動手、出事再處理；Rust 像開工前有嚴格的施工規範與檢查。這不代表 Rust 能保證「任何程式都無錯」，但它能在安全 Rust 的範圍內排除一大類記憶體安全錯誤。

> **從食譜到機器指令** <!-- callout:analogy -->
>
> 食譜是原始碼（source code），把食譜轉成機器能執行的指令是編譯（compile），最後按照指令做事是執行（run）。CPU 是實際執行機器指令的處理器；作業系統協助載入程式與分配資源。食譜比喻不能代表所有語言都要先產生獨立執行檔。

## 三、先認識工具：rustup、rustc、Cargo

rustup：工具鏈管理員，通常用來安裝、切換和更新 Rust 版本。

rustc：Rust 編譯器，把 Rust 程式碼轉為目標平台的可執行檔或函式庫。

Cargo：專案管家，負責建立專案、呼叫編譯、管理外部套件及執行常用工作流程。Cargo 不是另一個 Rust 語言。

比喻：rustup 像管理施工設備的採購與更新；rustc 是製作產品的主機；Cargo 是安排材料與工作流程的工頭。

三個最基本的檢查指令：

```bash
rustc --version
cargo --version
rustup --version
```

如果使用非 rustup 的安裝方式，前兩個可能存在而 rustup 不存在；這不必然表示 Rust 壞掉。

## 四、安裝與工作環境

電腦（Windows）：到官方安裝頁取得 rustup-init，按指示安裝。如提示缺少 MSVC 連結工具，依官方說明安裝 Visual Studio C++ Build Tools。安裝後重新開啟 PowerShell，再執行 rustc --version 與 cargo --version。

Linux／macOS：可依 Rust 官方安裝指示使用 rustup；Linux 也可能需要 C 編譯工具與連結器。Ubuntu 常見準備命令如下：

```bash
sudo apt update
sudo apt install build-essential curl
```

官方 rustup 安裝入口：<https://rustup.rs/>

手機（POCO／Android）：如果你想直接在 Termux 原生環境練習，Termux 套件倉庫通常可安裝 Rust。先更新，安裝後檢查版本：

```bash
pkg update
pkg install rust
rustc --version && cargo --version
```

注意：Termux 原生環境、Ubuntu PRoot 與 Windows 是不同的執行環境，不要混用它們的安裝指令。你已有 Ubuntu PRoot 的話，可在其內按 Ubuntu 路線配置；不要直接假設原生 Termux 的執行檔可在 PRoot 中共用。

## 五、第一次程式：Hello, world!（對照 1.2）

### 1. 先知道終端機在做什麼

終端機不是編譯器，它只是輸入指令、查看結果的介面。pwd 顯示目前資料夾、ls 列出檔案、cd 切換資料夾、mkdir 新增資料夾。先確定自己在哪個目錄，再編譯程式。

Linux／macOS／Termux：建立一個練習用資料夾。

```bash
mkdir -p ~/rust-learning/day01
cd ~/rust-learning/day01
```

用文字編輯器建立 main.rs，內容如下：

```rust run
fn main() {
    println!("Hello, Rust!");
}
```

### 2. 每個符號代表什麼？

fn：告訴編譯器「我要定義一個函式」。函式可以視為一組具名字、可被執行的步驟。

main：這個可執行程式的起始函式名稱；對現在的最小範例而言，執行程式會從它開始。

()：放函式參數的位置；空括號表示沒有參數。

{ }：包住函式本體，指定哪些敘述屬於 main。

println!("Hello, Rust!")：輸出一行文字。引號包住的是字串；驚嘆號 ! 表示 println! 是「巨集」，不是普通函式。巨集的深入原理留到後續章節。

;：在這裡用來結束這個敘述。請先不要把它理解為「每行都必須有分號」；Rust 的語句與運算式有更細緻的規則，之後會學。

縮排：大括號決定程式區塊，縮排主要讓人容易閱讀；慣例使用四個空格。

### 3. 親手編譯與執行

```bash
rustc --edition=2024 main.rs
./main
```

預期結果：Hello, Rust!

Windows PowerShell：同樣先用 `rustc --edition=2024 main.rs` 編譯，改用 .\main.exe 執行。Rust 原始碼不等於執行檔；請觀察編譯後新增的檔案。

理解 ./main：./ 表示「目前目錄下」，main 是要啟動的執行檔。與直接打 main 不一定等價，因為 Shell 會依 PATH 尋找命令。

> **第一個程式的四個部分** <!-- callout:walkthrough -->
>
> 讀第一個 Hello Rust：`fn main()` 是入口，`{}` 包住步驟，`println!` 顯示一行字，雙引號內是要顯示的文字。先只改引號裡的內容，再重新編譯，觀察輸出；先不需要知道巨集如何展開。

## 六、為什麼還需要 Cargo？（對照 1.3）

只寫一個 main.rs 用 rustc 很簡單；但程式長大後會有多個檔案、外部套件、建構設定、測試、不同版本依賴。若每次都自己拼出完整編譯命令，既耗時也容易出錯。Cargo 就是將這些工作標準化。

### 1. 建立新專案

```bash
cd ~/rust-learning
cargo new hello_cargo
cd hello_cargo
```

Cargo 會建立 src/main.rs 與 Cargo.toml，通常也建立 .gitignore；是否初始化 Git 視目前目錄與設定而定。

專案概念圖：

```text
hello_cargo/
  Cargo.toml       ← 專案的基本說明與依賴設定
  Cargo.lock       ← 建構後記錄依賴的確切版本
  src/main.rs      ← 程式入口的原始碼
  target/          ← 建構輸出，執行後才出現
```

Cargo.toml 使用 TOML 格式。它像一張物料與專案資訊表，包含 package 名稱、版本、Rust edition 以及 dependencies（外部套件依賴）。

不要死背 edition = "2021"：繁中教材以 Rust 2021 為基礎，而新版本 Cargo 可能生成 2024 edition；edition 是語言相容性規則的選擇，不等於 rustc 版本，也不是專案的版本號。今天保留工具產生的值即可。

### 2. 四個常用指令

```text
cargo run       ← 必要時先編譯，再執行
cargo build     ← 建立執行檔但不啟動
cargo check     ← 檢查程式能否通過主要編譯檢查；通常較快，不產生一般可執行檔
cargo fmt       ← 依 Rust 標準風格排版程式碼
```

cargo run 會先處理建構工作，而不是把 .rs 文字直接交給 CPU 執行。這是理解 Rust 工作方式的重要分界。

## 七、常見錯誤：刻意犯一次，才知道怎麼修

錯誤 A：把 println! 寫成 println。結果不是單純少一個裝飾符號，而是從巨集呼叫變成無法解析的函式呼叫。

錯誤 B：忘記字串的結尾引號。編譯器無法判斷文字到哪裡結束。

錯誤 C：在 Cargo.toml 的上層資料夾輸入 cargo run。Cargo 需要找到目前專案的 manifest；先用 pwd、ls 確認是否在專案目錄。

錯誤 D：看到編譯錯誤就直接重跑執行檔。請記得舊的執行檔不會因原始碼變更而自動更新；必須重新成功編譯。

除錯基本順序：讀取第一個有意義的錯誤訊息 → 確認檔名與位置 → 找到錯誤行及附近符號 → 修改 → 再執行 cargo check 或 cargo run。

> **改了程式，為什麼還是舊畫面？** <!-- callout:warning -->
>
> 如果修改了原始碼卻看到舊結果，先確認是否重新編譯、是否執行正確檔案。編譯錯誤表示這次沒有成功產生新結果，不代表電腦或 Rust 安裝一定壞了。

## 八、今日實作：先自己做，再看參考答案

練習 1｜改寫輸出。將 Hello, Rust! 改成自己的學習宣言，例如「今天開始學 Rust」，重新編譯並執行。

練習 2｜連續輸出。只用今天學到的 println!，在終端機印出三行：「第一行：學會原始碼」「第二行：學會編譯」「第三行：學會 Cargo」。

練習 3｜建立第二個 Cargo 專案 day01_practice，修改 src/main.rs，執行 cargo check、cargo build 與 cargo run，說出三者的差別。

練習 4｜不看上文回答：為什麼修改 main.rs 以後，直接執行舊 ./main 不會看到新內容？

練習 5｜找到下面程式所有明顯的語法問題：

```rust error E0765
fn main() {
    println("Hi Rust!)
}
```

## 九、參考答案與觀念檢查

練習 1：修改雙引號裡的文字即可，然後重新 rustc main.rs 或 cargo run（依你使用的專案種類）。

練習 2：

```rust run
fn main() {
    println!("第一行：學會原始碼");
    println!("第二行：學會編譯");
    println!("第三行：學會 Cargo");
}
```

練習 3：check 著重快速檢查；build 編譯生成執行檔；run 在必要時建構並執行。

練習 4：修改的是原始碼；執行檔是上次編譯的結果。未重新編譯，兩者內容就不同步。

練習 5：println 後缺少 !，字串缺少結尾雙引號；修正為 println!("Hi Rust!");。這裡可加分號以表達敘述。純粹修正巨集名稱與引號後，末尾的 `println!(...)` 也能作為回傳 `()` 的尾端運算式，因此分號在這個位置是可選的。

## 十、今日驗收（自己打勾）

□ 能用自己的話描述「原始碼 → 編譯器 → 執行檔 → CPU」

□ 能說出 fn、main、()、{}、println!、引號、分號的用途

□ 能在所選環境確認 rustc 與 cargo 可用

□ 能用 rustc 及 cargo 分別執行 Hello, World

□ 能讀懂最基本的編譯錯誤並修正

以上五項大致能做到，才進入 Day 02。接下來依原書第 2 章建立「猜數字遊戲」；它會刻意提前碰到變數、輸入與外部套件，我會在必要處先補原理，不會假設你已經學過。

> **讓電腦說兩句不同的話** <!-- callout:practice -->
>
> 不看答案，說明 `rustup`、`rustc`、`cargo` 各管什麼。將 Hello Rust 改成自己的兩行訊息，用 `cargo run` 執行，再解釋為什麼 `cargo check` 不會顯示程式自己的輸出。

## 十一、原始教材與版本提醒

繁中版第 1 章：<https://rust-lang.tw/book-tw/ch01-00-getting-started.html>

1.1 安裝：<https://rust-lang.tw/book-tw/ch01-01-installation.html>

1.2 Hello World：<https://rust-lang.tw/book-tw/ch01-02-hello-world.html>

1.3 Hello Cargo：<https://rust-lang.tw/book-tw/ch01-03-hello-cargo.html>

官方最新版英文教材：<https://doc.rust-lang.org/book/ch01-00-getting-started.html>

Rust 官方安裝：<https://rustup.rs/>

註：這是一份依原書章節順序重新編寫、加入底層解釋與練習的原創教學筆記；不是原書全文複製。繁中版基於 Rust 2021，實作請以目前安裝的穩定版工具輸出為準。
