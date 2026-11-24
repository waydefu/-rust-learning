# A05｜從語言基礎延伸到真實領域

注意：各領域依賴的框架更新很快。本文件建立原理、學習順序、驗收方法與權威來源，不將示意工程當成已部署產品。

## 一、怎麼判斷學習路線？

先完成所有權、struct/enum、Result、Trait、集合、測試（第4–11章）。再依想解決的問題選技術：瀏覽器運算用 WebAssembly；微控制器用 Embedded；高併發 I/O 用 async 後端；桌面/遊戲工具先從純計算核心切入。

關鍵原則：先學 Rust 的資料與安全模型，再學框架；否則一旦框架 API 改版就容易失去判斷基準。

## 二、WebAssembly：為何能在網頁使用 Rust？

Rust 可編譯成 Wasm 模組，讓瀏覽器中的沙盒執行環境運算。WebAssembly 不是取代全部 JavaScript 的通用 DOM 控制器；與 JS/瀏覽器 API 互動常需 wasm-bindgen、web-sys 等綁定工具。

設計分工：JavaScript/UI 處理頁面、使用者互動與 DOM；Rust/Wasm 負責適合的計算核心。跨邊界的資料序列化與記憶體交換有成本，未必所有小功能都因 Wasm 更快。

進階練習：做一個 Rust 的純數學計算函式 → 在 native cargo test 驗證 → 編譯到 wasm32 目標 → 用 wasm-bindgen 包成 JS 可呼叫 API → 在手機瀏覽器驗證。不可跳過實際目標平台建構和網頁整合。

官方參考：<https://rustwasm.github.io/docs.html>

## 三、ESP32 與嵌入式：沒作業系統怎麼辦？

微控制器可能只有少量 RAM/Flash，沒有完整 std 或作業系統服務；no_std 使用核心語言與 core library，與一般桌面 Linux 應用是不同運作環境。

ESP32 不是單一 CPU 家族：某些型號是 Xtensa，某些型號是 RISC-V。不同晶片支援的 Rust toolchain、target、HAL、Wi-Fi、工具鏈可能不同；先確認具體晶片與開發板，再照 Espressif 官方文件配置。

典型硬體流程：讀取 GPIO/UART/I2C/SPI → 驗證感測器資料 → 轉成 enum 代表狀態 → 異常檢查 → 更新 LED、蜂鳴器或畫面。儘量把「硬體驅動」和「判斷邏輯」分開，後者可在電腦上單元測試。

官方參考：<https://docs.espressif.com/projects/rust/book/>

嵌入式 Rust：<https://doc.rust-lang.org/stable/embedded-book/>

## 四、遊戲與圖形

學習先拆成核心邏輯與呈現：位置、速度、碰撞、角色狀態是純資料與函式；圖形後端則牽涉 GPU API、Shader、資源生命週期、同步與平台限制。

純 Rust 可先寫 fixed timestep 的遊戲更新器，並用測試檢查能量守恆、碰撞判定、狀態合法性；再考慮引擎或渲染函式庫。不要把 Rust 的「執行快」等同「GPU 畫質自然變好」，後者取決於演算法、GPU、資產品質與渲染設計。

## 五、後端與非同步

後端核心涉及 HTTP、資料庫、序列化、驗證、身份認證、錯誤處理、網路安全、觀測性。Rust async 是協調大量 I/O 工作的工具，但需要合適 runtime，不會自動修復 SQL injection、認證錯誤或設計缺陷。

建議順序：先做簡單 TCP → 分離處理器邏輯 → 學 Tokio / async → 用成熟 HTTP 框架 → 資料庫交易 → logging / metrics → 性能與安全測試。

## 六、FFI 與外部原生函式庫

Rust 可以呼叫 C ABI 函式，或供其他語言呼叫；重點是 ABI 相容、資料布局、資源釋放、記憶體有效性及執行緒。需要 unsafe 時，先寫明安全條件再做小範圍包裝，切勿把 unsafe 當作編譯錯誤的快速消音器。

## 七、延伸領域的共同驗收法

把每個專案分成「純 Rust 可獨立測試的核心」和「平台特定的 I/O 與外部 API」。前者寫單元測試；後者寫整合測試與實機驗收。做到任何一個平台壞掉時，可以先判斷是程式邏輯還是外部環境問題。

## 八、練習題

從一個溫度感測器資料流出發：先定義 SensorReading 與 AlarmState，再寫 fn evaluate(...) -> AlarmState，加入測試；之後才決定輸入來自 ESP32 UART、CLI 檔案或網頁 API。

進階：為同一個演算法製作 CLI 與 Wasm 兩種外殼，證明核心邏輯可以共用。

Rust Platform Support：<https://doc.rust-lang.org/rustc/platform-support.html>
