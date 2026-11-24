# 第14章｜一個專案為何不只是一堆 .rs 檔案？

> **來龍去脈**
>
> 你的遊戲在自己電腦能跑，朋友下載後卻缺少套件或使用了不同版本。如果每個人靠手動記憶安裝步驟，結果很容易不同。Cargo 把專案需要的材料、編譯方式和測試流程記錄下來。

## 一、根本問題：程式能在其他電腦重現嗎？

獨立編譯 main.rs 很簡單，但真實產品需要依賴、測試、不同執行環境、建構模式、文件與發佈。Cargo 把這些工作變成可重複的工程流程。

比喻：Cargo.toml 是「希望使用哪些材料與規格」；Cargo.lock 是一次解析後更精確的材料清單；target/ 是建築工地的產物。這只是示意：實際依賴來源、特性旗標及目標平台都可能影響建構。

> **生活比喻**
>
> `Cargo.toml` 像專案訂購單，描述希望使用的套件版本及設定；`Cargo.lock` 記錄一次解析得到的確切依賴版本；`target/` 放建構產物。Rust 的 edition 是語言相容性規則，不是你下載的套件版本。

## 二、依賴版本如何運作？

```toml
[package]
name = "my_tool"
version = "0.1.0"
edition = "2024"

[dependencies]
# 依需求執行 cargo add 套件名稱，別任意貼過時版本
```

Rust Edition 控制語言的相容性與慣例，不等同編譯器版本。SemVer（語意化版本）以 major.minor.patch 表達相容性承諾；但 0.x 的版本升級規則有不同慣例，Cargo 的版本需求也不等同於「完全鎖死一版」。

Cargo.lock 記錄解析出的依賴版本；應用程式通常將它納入版本控制，以提升建構可重現性。公開函式庫的 Cargo.lock 也可能存在，但下游使用者會依其自己的依賴解析流程決定版本。

> **程式拆解**
>
> 先掌握工作流程：`cargo check` 做主要編譯檢查，`cargo build` 建立產物，`cargo run` 建構並執行，`cargo test` 建構並執行測試。`--release` 選擇發行用 profile，通常開啟較多最佳化，但不是每個程式都因此變快。

## 三、四個建構指令的差異

cargo check：檢查編譯而通常不產生一般可執行檔；cargo build：建構 debug 組態；cargo build --release：啟用 release profile 的最佳化；cargo test：編譯並執行測試。

cargo fmt：格式化；cargo clippy：提供常見正確性與慣例檢查；cargo doc --open：產生文件。不要把 clippy 所有提醒都視為絕對錯誤，應理解建議依據。

## 四、為什麼 release 和 debug 性能不同？

最佳化可能消耗更多編譯時間並改變產出的機器碼；debug 預設更注重開發速度與除錯便利。效能比較應確定輸入、編譯設定、硬體、測量方法一致，避免把「第一次建構時間」錯當執行效能。

> **常見陷阱**
>
> 相同 Cargo.lock 不代表所有電腦一定產生完全相同的二進位檔；Rust 工具鏈、目標平台、系統函式庫也會影響結果。feature 是套件可選能力，不是能無條件互斥地開關任意功能。

## 五、工作空間（Workspace）

當產品拆成 CLI、共通函式庫、伺服器等多個 package，可使用工作空間統一管理。

```toml
[workspace]
members = ["crates/core", "crates/cli"]
resolver = "3"
```

此範例適用 Rust 2024 Edition 的常見 workspace 設定；具體 resolver 取決於工具鏈與需求。

在 workspace root 執行 cargo test --workspace，可測試全部成員。共用 target/ 減少重複工作，但也可能增加跨成員建構耦合。

## 六、features 與條件編譯

Cargo feature 允許依需求啟用程式能力，例如某些後端或可選依賴；feature 常是「加法式」，因此應避免把互斥功能當成任何組合都合法。

cfg! 或 #[cfg(...)] 用來決定程式段落是否依條件生效。feature 不是執行時任意切換的 UI 開關，通常需要重新建構才改變可用程式碼。

## 七、發佈前的契約

先檢查 README、授權、API 文件、測試、cargo package 與 cargo publish --dry-run，再考慮發佈到 crates.io。公開 crate 的名稱與版本持續存在於生態系，須重視 API 相容性與供應鏈風險。

cargo install 通常用於安裝公開提供的二進位工具；它不是替專案新增函式庫依賴的操作，新增依賴用 cargo add。

## 八、供應鏈安全與維護性

引入第三方 crate 等於信任其維護與發佈流程。比較是否真的需要、更新頻率、授權、審核、已知漏洞、傳遞依賴和 features。需要時配合 cargo audit 等工具，但掃描不能取代程式碼與版本鎖定的設計審查。

> **動手驗收**
>
> 建立小專案並依次執行 check、build、run、test，觀察輸出差異。找出 Cargo.toml 與 Cargo.lock，解釋若要重現一次建構，除了依賴版本還需要記錄什麼。

## 九、驗收

題1：說明 Cargo.toml 與 Cargo.lock 的分工。

題2：比較 cargo check / build / test / run 的不同任務。

題3：建立一個包含 core 與 cli 的 workspace，由 cli 使用 core 公開函式。

題4：辨認 crate/package/module/workspace 的層次差異。

參考：<https://doc.rust-lang.org/stable/book/ch14-00-more-about-cargo.html>
