# A04｜實作環境與可靠開發流程

## 一、為什麼一樣的程式不同裝置會有差別？

Rust 原始碼可重複編譯到不同平台，但可執行檔與其目標架構、作業系統 ABI、動態函式庫、連結器有關。Linux x86_64 編譯的檔案，不能假定在 Android ARM64 原生 Termux 上直接執行。

比喻：同一份設計圖可以交給不同工廠製造，但成品的電源插頭、尺寸與接線規格必須符合實際使用環境。

## 二、標準桌面環境

Windows：優先依 <https://rustup.rs/> 安裝官方工具鏈，依提示處理 MSVC C++ Build Tools。PowerShell 執行 rustc --version、cargo --version、rustup --version。

Linux / macOS：依 rustup 官方步驟安裝，Linux 常需 C linker 和基礎 build 工具。

確認：cargo new playground；cd playground；cargo run；cargo fmt；cargo clippy；cargo test。

專案變更前先讓 cargo check 通過，然後再做整合測試與 release 建構。cargo new 預設 edition 取決於目前工具鏈。

## 三、Android / POCO / Termux 原生路線

Termux 是 Android 上的 Linux 類使用者空間，但並不等於 Ubuntu；不要直接把 Ubuntu apt 套件和 Termux pkg 套件混用。

```bash
pkg update
pkg upgrade
pkg install rust
rustc --version
cargo --version
cargo new hello_phone
cd hello_phone
cargo run
```

Termux 套件倉庫的 Rust 不一定使用 rustup 管理。若 rustup --version 不存在，應先判斷安裝途徑，不是立刻判定失敗。

手機熱管理、記憶體壓力、儲存空間與電池可能影響大型依賴編譯。先從小型 CLI 專案練習，避免一開始把大型 Web server + OpenSSL + graphics crates 全部裝上去。

## 四、Ubuntu PRoot 是另一套環境

若使用 Termux 內的 Ubuntu PRoot，就在 Ubuntu 裡依該系統套件與 rustup 安裝 Rust，並注意 ARM64 目標及 linker。PRoot 的 Linux userspace 和 Android Termux 原生環境有不同的二進位相容性邊界。

Ubuntu 常見準備：

```bash
sudo apt update
sudo apt install build-essential curl pkg-config
```

使用 Rust 官方安裝方法前先確認 shell、PATH 與網路權限。不要同時把不同位置的 cargo、rustc 和工具鏈 PATH 指到混亂的版本。

## 五、編輯器與便利工具

VS Code 或支援 LSP 的編輯器可搭配 rust-analyzer。遇到某個編輯器的提示和 cargo check 不一致，先用實際專案工具鏈編譯確認。

cargo fmt 統一風格；cargo clippy 指出潛在問題；cargo doc 產生 API 文檔；cargo test 驗證行為；cargo tree 可分析依賴樹；cargo metadata 可讓工具理解專案結構。

## 六、Git 與可重現性

把 src/、Cargo.toml、相關 Cargo.lock、README、測試放進版本控制，通常不要提交 target/ 的大量建構產物。發現工具問題先記錄 rustc --version、cargo --version、rustc -vV、OS/架構、錯誤訊息與最小重現程式。

## 七、效能與除錯的循環

先做正確版 → 寫測試 → 檢查警告 → 用 release 模式做相同輸入基準測試 → 找瓶頸 → 最小變更 → 回歸測試。不可以「更少行」推論效能更快，也不能用 cargo check 的速度判斷執行效能。

## 八、手機端驗收

在原生 Termux 建立 hello_phone 並跑出結果；在 Ubuntu PRoot 分別檢查工具鏈；區分兩者編出的檔案存放路徑與可執行環境；能判斷 PATH 錯誤、linker 錯誤、版本相容錯誤的差別。

Rustup：<https://rustup.rs/>

Termux 套件：<https://github.com/termux/termux-packages>

Cargo 指令：<https://doc.rust-lang.org/cargo/commands/index.html>
