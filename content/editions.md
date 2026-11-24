# A03｜Rust 2024 Edition 與繁中舊版教材對照

## 一、先釐清三個不同「版本」

Rust 語言工具鏈版本（例如 rustc 1.98）反映編譯器發佈。Rust Edition（例如 2021、2024）是專案在 Cargo.toml 選擇的語法相容性規則。Crate 版本（例如 0.1.0）是自己的套件版號，與前兩者沒有直接對應。

比喻：rustc 版本像教科書印刷更新；edition 像選擇使用哪一套考試規則；crate version 是你的作品發行編號。新編譯器通常能編譯較舊 edition 的程式，這是 Rust 控制相容性的重要設計。

## 二、為什麼要有 Edition？

語言進化時，若每個關鍵字或語義改動都強迫所有舊專案立即修改，生態系就會分裂。Edition 讓原有程式可以繼續依舊規則編譯，願意升級的專案再分階段處理必要改動。

不同 edition 的 crate 仍可以互相成為依賴；不必要求整個依賴樹同一天升版。

## 三、新舊書章節最大的差異

繁中舊版：第17章物件導向、第18章模式、第19章進階、第20章 Web Server。

新版官方：第17章新增 Async/Await/Future/Stream，第18章物件導向、第19章模式、第20章進階、第21章 Web Server。

因而本資料夾將課程完整編為第1–21章；不需要強迫把原繁中書所有內容重編號，閱讀時透過這份對照定位。

## 四、Rust 2024 的重要語意差異

gen 成為保留關鍵字，若舊碼用 gen 當變數或函式名，可能需改名或使用 raw identifier r#gen。

extern 區塊在 2024 edition 要明確寫 unsafe extern，提醒跨語言函式宣告有安全契約。

部分過去可安全呼叫的環境變數等操作，在 2024 Edition 需要 unsafe，因修改整個程序的全域環境可能與其他執行緒衝突。

unsafe 屬性如 no_mangle、export_name、link_section 的使用方式也有 edition 變化；不要不看文件就直接複製舊教學。

語義變化不只這些，還有 RPIT lifetime capture 規則、臨時值作用域、標準庫 Prelude 與 Cargo resolver 等更新。進階遷移時應逐項對照官方 Edition Guide，不建議把一張簡表當成全部。

## 五、依賴套件新版 API 很容易踩雷

舊版猜數字教學常出現 rand::thread_rng().gen_range(...)，新版 rand API 已演變，不能把舊用法當永遠正確。請先執行 cargo add rand，再依目前鎖定版本查看 docs.rs 文件。

版本升級不是單純把數字改大：依賴的 breaking change 可能要求修改 import、方法名稱與 Feature Flag。每次升級都應執行 cargo check、cargo test 和相關整合測試。

## 六、正確的升級程序

1. 提交現有可工作的程式碼與測試。

2. 讀官方 Edition Guide，先對所有警告理解成因。

3. 使用 cargo fix --edition 協助處理可機械式改動，檢查修改內容。

4. 更新 Cargo.toml 的 edition 欄位。

5. 執行 cargo fmt、cargo check、cargo clippy、cargo test。

6. 對關鍵行為與效能做回歸驗收，不能把編譯成功當成完成遷移。

## 七、如何查 Rust 的權威資料？

學習觀念：The Rust Programming Language。

精確語法與語義規格：Rust Reference。

標準庫 API：std 文檔。

版號相容性：Edition Guide、Rust Release Notes。

套件 API：docs.rs 及各 crate 的 CHANGELOG。

專案工程：Cargo Book。

底層 unsafe 規則：Rustonomicon，但需留意其部分內容是進階與可能演進中的資料。

## 八、練習

題1：用自己的話解釋為何 2021 edition 可以在新 rustc 上編譯。

題2：把舊程式名 gen 改成合法 2024 名稱，解釋 raw identifier 的目的。

題3：對一個小專案執行 cargo fix --edition，逐一閱讀變更。

驗收：能區分工具鏈／edition／crate 版號；知道原繁中書章節和新版的差別；遇到相衝突教學時知道先檢查哪一份來源。

Rust 2024 Edition Guide：<https://doc.rust-lang.org/edition-guide/rust-2024/>

最新版教材：<https://doc.rust-lang.org/stable/book/>

繁中教材：<https://rust-lang.tw/book-tw/title-page.html>

Cargo Book：<https://doc.rust-lang.org/cargo/>
