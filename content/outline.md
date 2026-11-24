# Rust 語法｜完整課程總目錄（2024 Edition，21章）

這是課程的唯一入口。按官方最新版 21 章順序由零開始；繁體中文版作為概念參考，遇到版本差異以 Rust 官方文件確認。

## 學習方式與驗收規則

每章依序閱讀：問題從何而來 → 為什麼如此設計 → 內在機制 → 直覺比喻及其限制 → 逐行程式 → 反例 → 練習與答案線索 → 自我驗收。先能解釋每一項的因果，才算完成。

教材「已建立」不代表學習者「已完成」。尤其第4章（所有權）、第10章（泛型與生命週期）、第15–17章（智慧指標／並行／非同步）應拆成多次練習。

## 第1章｜開始入門（既有 Day 01）

第01章｜Day 01：Rust、編譯器、Cargo、Hello World

## 第一階段｜基礎核心（第2–7章）

第02章｜猜數字遊戲：輸入、型別與流程

第03章｜變數、型別、函式與控制流程

第04章｜所有權、借用與切片：記憶體核心

第05章｜Struct 與方法：資料建模

第06章｜Enum、Option、match 與狀態

第07章｜Cargo、crate、模組與公開 API

## 第二階段｜集合、抽象與工程（第8–14章）

第08章｜Vec、String、HashMap 與 UTF-8

第09章｜panic、Result 與錯誤傳播

第10章｜泛型、Trait、生命週期

第11章｜自動化測試、斷言與規格

第12章｜CLI 與檔案搜尋實作

第13章｜閉包、迭代器與惰性計算

第14章｜Cargo、套件、Workspace 與發布

## 第三階段｜系統與進階（第15–21章）

第15章｜智慧指標與共享所有權

第16章｜執行緒、通道、Mutex、Send/Sync

第17章｜Async、Await、Future、Stream（新版新增）

第18章｜物件導向理念、Trait Object、Typestate

第19章｜模式配對、解構與守衛

第20章｜Unsafe、FFI、進階 Trait 與巨集

第21章｜TCP / HTTP 多執行緒伺服器

## 深入理解與延伸研究（A01–A08）

A01｜記憶體與編譯器全圖解

A02｜編譯錯誤診斷圖鑑

A03｜Rust 2021 與 2024 Edition 差異

A04｜Windows／Linux／Android Termux 與工具鏈

A05｜Wasm／ESP32／後端／遊戲／FFI 延伸

A06｜階段實戰與驗收

A07｜語法與符號字典

A08｜綜合推理題庫與解答

## 官方權威來源與使用邊界

[Rust 官方最新版 The Rust Programming Language](https://doc.rust-lang.org/stable/book/)

[Rust 繁體中文社群翻譯](https://rust-lang.tw/book-tw/)

[Rust Reference（精確語言語義）](https://doc.rust-lang.org/reference/)

[Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/)

[Rust 標準函式庫](https://doc.rust-lang.org/stable/std/)

本套文件皆為自行編寫的教學與示例，不直接重製書籍全文。這是完整主幹與重要延伸主題的第一套教材，不等同語言規格的每個細節、所有 crate 或生產級系統的完全指南。外部套件 API 會隨版本改變，應以實際專案版本核實。

教材建立日期：2026年11月22日。課程進度仍由實際操作與理解驗收決定，不因文件齊全就視為課程已完成。
