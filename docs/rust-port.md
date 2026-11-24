# Rust 建置器移植紀錄

## 基準與決策

以已合併的 `4ed054535e0b3ffb423432be2b899de838b988f3` 為基準。PR #8 的 [CI](https://github.com/waydefu/-rust-learning/actions/runs/45375278739) 通過 16 個原生工具測試、112 段 Rust 教材範例及 9 個瀏覽器測試；34 頁文件模板由 Rust 生成，正文 30/31 篇相容，第 13 章因 URI 誤判回退。

參考 [rust-anything v4.0.0 全應用程式流程](https://github.com/erkamyaman/rust-anything/blob/v4.0.0/docs/whole-apps.md)：記錄原版行為，移植單元，重播，確認無回退後才移除舊路徑。工具僅是移植測試器，不是程式翻譯器。已記錄案例相符不代表所有可能輸入等價。

- 教材與正式建置維持 Rust 1.85.1；rust-anything v4.0.0 使用驗證過 SHA-256 的 Linux 預編譯 release，不加入網站 Cargo 依賴，不執行 setup。
- 保留 HTML/CSS 與必要瀏覽器 JS；不為了 Rust 比例引入 Wasm。保持網址、錨點、搜尋 schema、localStorage、HackMD 與 Pages 不變。
- 原版輸出是移植對照，不是錯誤永遠不能修正。第 13 章的泛型改為行內程式碼，是明確的教材格式修復。

## 單元地圖

| 單元 | 狀態 | 驗收方式 |
| --- | --- | --- |
| 教材預檢、搜尋索引、本機預覽 | 已合併 | 原生測試、既有 CI |
| 共用 HTML 文件模板 | 已合併 | 34 頁逐字對照 |
| Markdown 正文 | 31/31 已驗證、已部署 | CI 強制 31/31，禁止回退 |
| 靜態產物寫入 | 已驗證、已部署 | rust-anything record/replay 與完整產物逐位元組對照 |
| 章節標題元資料、Unicode 錨點 | 已驗證、已部署 | 31 篇逐項比對、Unicode 邊界案例；catalog 讀取與搜尋全文尚在 JS |
| 導覽、首頁、互動教材模板 | 側欄導覽已部署；首頁、資源與互動模板 PR CI 通過 | 34 頁 DOM 與瀏覽器測試 |
| 語法高亮、範例抽取、網站檢查 | 原生規則與執行器已提交；高亮零回退驗收以 PR #12 最新 CI 為準 | fenced code、診斷、測試與輸出契約 |
| Rust CLI 總入口 | 已移植、已部署（PR #13） | 完整 build/check/dev 行為錄製與重播 |

## 第一個 rust-anything 單元：site-write

`scripts/migration/legacy-site-write.mjs` 將原版 build.mjs 的 rm/mkdir/cp/writeFile 序列移入測試專用 adapter；CLI 封包及輸入拒絕規則為新加入的明確契約，不是聲稱舊內部函式已支援任意非法輸入。Rust 新寫入器先建立完整暫存樹，再替換舊 dist，並另外驗證二進位資源、符號連結和失敗保留舊產物。

`write-cases.mjs` 產生 14 個明確案例（含繁體中文、Emoji、空檔、巢狀路徑、舊檔移除、重複及路徑遍歷）。`replay-site-write.sh` 在隔離目錄執行上游工具：

1. app record 以 Node 參考程式執行每案兩次，不接受不穩定、噪音或逾時案例。
2. app replay 重播原版，全部必須通過。
3. app replay --build ./site-write 重播 Rust，全部必須通過，不允許跳過或回退。
4. 原始 cases、JSON 結果與差異報告作為 CI artifact，不發布到網站。
5. VERIFY_SITE_WRITE=1 對根路徑與 Pages 子路徑生成的完整網站樹逐檔比對原版 writer。不是整個建置器已純 Rust 的證據。

本輪不執行 app lift：它需要完整入口與所有單元通過，尚未達成。不得將測試尚未執行、或僅有移植程式碼提交，記為已完成驗收。

## 第五階段：標題、錨點及導覽

新基準 `0e45188a8ddfb7b286d546a9ff7e9ca3e0ede9f0` 的 [main CI 第二次執行](https://github.com/waydefu/-rust-learning/actions/runs/45376847485) 完成發布與線上驗證。上一輪 14/14 原版及 Rust 重播、31/31 正文、43 個輸出檔案比對、112 段 Rust 範例及 9 個瀏覽器測試通過。

Rust 直接從 Markdown 抽出 H1 及各節標題，以 Unicode NFKC、完整小寫、General Category 的 Letter/Number 篩選與每篇重複計數生成錨點。固定依賴版本與 crates.io checksum，保留原 JS 計算為相容性基準；任何標題、層級或錨點差異都停止建置。Unicode 測試包含全形、組合字、Greek final sigma、Emoji、非 ASCII 字母、重複和空標題。保留既有重複錨點演算法，不聲稱已修復所有可能的後綴碰撞。Node 仍提供原 Markdown token offset、catalog JSON 讀取、搜尋全文。

Rust 生成 34 頁桌面與手機共用的課程側欄，逐字比對舊版導覽，再由 Rust 文件模板放入頁面。原 JS 模板暫留驗證；所有路由、學習進度、CSS 和瀏覽器腳本不變。現有 rust-anything 寫入器重播仍在 CI 中，本輪新增單元使用原版直接差異驗證，未聲稱已完成 rust-anything 全應用程式 lift。

PR #10 已合併；PR #11 拆分 Pages 產物上傳與部署 job，main `3d983a3` 的 [Actions](https://github.com/waydefu/-rust-learning/actions/runs/45379517169) 首次執行即成功，含線上版本檢查。

## 第六至九階段：單一 PR #12

基準 main `3d983a3` 已成功部署。依維護者要求，第六至九階段集中在 [PR #12](https://github.com/waydefu/-rust-learning/pull/12)，不拆分後續 PR。

- 第六階段：Rust 生成首頁、延伸資源、404、四組課程階段、21 張章節卡片、8 張資源卡片及共用頁尾。
- 第七階段：Rust 生成 31 頁閱讀正文外框、桌面及手機目錄、上下章導覽、網站章節索引、測驗、流程圖、所有權記憶體圖、完成區塊及 34 頁 page-data JSON。只接收結構化欄位、可信 SVG 與原生 Markdown 正文，不接收舊版頁面片段；瀏覽器 JS 不變。
- 第八階段：Rust 自行掃描 Rust、Shell、TOML 與純文字，產生高亮、複製按鈕及程式區塊。參考 highlight.js 11.12.0 的公開語法契約。highlight-v2 只接收原始程式碼與可信 SVG，舊版 HTML 不進入 Rust，僅供比對。任何差異拒絕建置，報告記錄各語言原生採用率及 fallback=0。這是現有教材與回歸測試的契約，不代表支援所有可能的語法輸入。
- 第九階段：Rust 從 Markdown 抽取 fenced code、驗證範例標注，接管 Rust 編譯、執行、預期診斷碼集合、Cargo 多檔案專案、逾時與暫存清理。Rust 網站驗證規則包含 catalog、來源檔案、標題層級、程式碼結尾、HTML 結構、教學方塊、站內連結與錨點、資源檔案、搜尋索引完整欄位與定位、渲染報告及 160 kB 預算。Node 保留 HTML5/URL/JSON 解碼及結構化資料橋接；不再執行正式檢查規則，舊執行器及檢查器只作 CI 對照。

### 已取得的歷史證據

| CI | 實際結果 |
| --- | --- |
| [#29](https://github.com/waydefu/-rust-learning/actions/runs/45380643836) | 125 段抽取、112 段 Rust 範例、10 瀏覽器測試、14/14 行為重播通過；高亮 6/125 |
| [#30](https://github.com/waydefu/-rust-learning/actions/runs/45381146453) | 32 個 Rust 工具測試、原生執行器 112/112，與舊版統計一致 |
| [#31](https://github.com/waydefu/-rust-learning/actions/runs/45381298048) | 原生網站檢查與舊版統計一致，完整 CI 通過 |
| [#32](https://github.com/waydefu/-rust-learning/actions/runs/45381465052) | 高亮 116/125，完整 CI 通過 |
| [#35](https://github.com/waydefu/-rust-learning/actions/runs/45382508912) | 124/125 高亮相容，首頁、資源與 404 三頁及 34 頁互動 JSON 比對通過，完整 CI 通過 |

唯一剩餘高亮差異是 testing.md 的 `super::*`。後續提交補齊 namespace 規則，改成零回退契約。網站檢查器新增七個暫時破壞產物的負例，兩套檢查器都必須拒絕，finally 還原原始位元組。

### 最終驗收條件

根路徑與 Pages 子路徑都必須通過 31/31 原生正文、全部 fenced code 原生高亮零回退、34 頁完整 HTML 與互動資料逐字對照、全網站檢查、Rust 範例與舊版統計對照、負例、行為重播、瀏覽器與本機預覽驗證。以 PR #12 最新提交的 Actions 成功紀錄為準；PR 未合併前不宣稱正式部署。

第十階段再整合純 Rust 建置總入口、消除 Node 的 catalog/搜尋全文/HTML5/URL/JSON 解析橋接，遷移圖示及互動資料來源，執行全應用程式行為驗證。現有 rust-anything 14 案僅為 writer 單元，不宣稱完成 app lift。

## 第十階段：純 Rust CLI 與完整行為對照

PR #12 已合併，main a51017e 的 Actions #40 首次發布成功。此 PR #13 新增 site-render 的 build/check/check-rust/dev 入口，讀取同一份 catalog.json、site-data.json 及原始教材，使用鎖定 serde_json、scraper/html5ever、url 取代 Node 解碼橋接；高亮、Markdown、模板、互動 JSON、索引、驗證規則與原生工具協作均由 Rust 執行。正式 Pages job 不再執行 npm ci，Node 只用於參考版及瀏覽器測試。CI 新增完整網站樹對照、錯誤路徑拒絕、原生檢查器正負例、Rust 範例對照及無 Node 的獨立驗證。根目錄和子路徑的原生輸出均須通過。PR #13 已合併，main `9cffa93` 的 [Actions](https://github.com/waydefu/-rust-learning/actions/runs/45386706262) 全部成功，並確認線上版本。rust-anything writer 原有 14 案重播保留。另以全 CLI 實際執行的黑箱觀察 adapter 錄製根路徑、Pages 子路徑、無效路徑、缺少教材、範例執行與本機 HTTP 預覽六案；每次重新複製真實課程樹，以全部輸出 SHA-256（報告 JSON 僅正規化 key order）、退出碼、保留舊產物、驗證摘要及 HTTP status/header/body 雜湊為觀察契約，原版和 Rust 皆須重播成功，無 fallback。adapter 只供驗收，不是正式 CLI 依賴。未使用上游 scaffold，故不宣稱執行 app lift。

### 動效與無障礙

首頁啟動提示使用 CSS 與既有瀏覽器 JS，在每個分頁工作階段至多播放一次；無遮罩、不阻擋點擊、不搶焦點，深層錨點與 reduced-motion 不播放。卡片、按鈕、閱讀頁首、對話框、測驗及記憶體圖採短時動畫，不延遲導覽或正文。無 JavaScript 時教材仍直接可讀。Playwright 驗證啟動提示、工作階段與減少動態效果。


## 後續品質維護
- 目前的原版對照是回歸基準，不證明所有新輸入正確；新增規則須搭配獨立正負例。
- `rust run` 可用自然程式註解 `// 預期輸出: 文字` 宣告確定性輸出，依序比對完整執行輸出（含換行，CRLF 正規化）；無註解仍只檢查結束狀態。不要用於亂數、耗時或互動輸出。
- 範例暫存工作目錄不是安全沙箱；只執行受信任的教材，勿在具有私密憑證的環境執行未審查 PR。
- 分支保護、已合併分支清理與實機測試需由維護者核准或提供相應環境。不要因移植完成就刪除仍用於驗收的 Node 基準。

### 品質稽核基線與修正

PR #14 首輪 [Actions](https://github.com/waydefu/-rust-learning/actions/runs/45397395188) 的功能與跨瀏覽器驗證成功；非阻擋品質 Job 日誌指出 Rustfmt 差異、六項 Clippy 診斷，以及 RUSTSEC-2026-0433、RUSTSEC-2026-0414、RUSTSEC-2026-0592。依該版 rustfmt diff 整理原始碼並修正診斷；pulldown-cmark 升為 0.13.6、url 升為 2.5.9，依 crates.io index 更新鎖定版本與校驗值，仍須以新的 locked build、原版對照及 RustSec 驗證為準。fxhash 的 RUSTSEC-2025-0057 為無修補版本的維護性警告，經 selectors/scraper 傳遞引入；後續評估替代版本與最低 Rust 版本相容性，不隱藏警告。

CI 的臨時 Cargo metadata 診斷（run 45398892481）實際產生鎖檔差異：url 2.5.9 的依賴清單新增 serde；已按該差異補齊並移除臨時 Job。正式驗證仍使用 --locked。

技術債清理階段：所有現有 `rust run` 範例都宣告明確 stdout 契約（含無輸出），stderr 不計入預期輸出；程式碼中的註解不改變執行語意。新版執行器限制環境變數及輸出量並在 Unix 逾時時終止程序群組，這些是防禦措施而非安全沙箱。

## PR #15：架構清理與固定參考

前端改成職責分離的 ES modules 與獨立動效/實驗室樣式；Rust 工具統一 workspace，正式流程直接呼叫函式庫；移植協定只留作固定參考相容入口。依賴 scraper 0.25.0 經 Cargo 實際解析移除 fxhash。歷史 Node 原始碼移出目前工作樹，保留在已全綠的 `b2c5abe32610f6f098e46ec2501c9514fed2066d`（[Actions](https://github.com/waydefu/-rust-learning/actions/runs/45400927384)）；`scripts/regression/verify-reference.sh` 在暫存目錄取出固定參考，搭配目前 HEAD 的 Rust 工具和內容，執行原有完整產物、負例及 rust-anything 重播。另有獨立 `check-native.mjs` 驗證可重現性與拒絕無效輸入。不宣稱固定參考能證明所有未來新輸入，也不在未確認的新 CI 上宣稱通過。

### 型別化原生流程

導覽、首頁、閱讀頁（教材分類、測驗、流程、所有權）、頁面資料、範例報告與網站驗證均改用具名 Rust 型別和直接呼叫。歷史 stdin 協定集中到 compat.rs；舊版 adapter 仍使用相同渲染及驗證實作，不在原生流程中重新編碼或解析欄位。保留固定歷史參考、原生負例、跨瀏覽器及品質關卡。

### 最終退役：固定輸出而非雙套實作

從成功的 Actions 45413171553 下載 artifact 14932370146，保留兩套 51 檔完整 SHA-256 清單作核准基準。歷史 stdin 協定、Node 參考下載/執行、搜尋及寫入器遷移 CLI 全部移出目前樹；舊版歷史證據留在 Git。原生正負例、Unicode fixture、瀏覽器及逐檔雜湊持續強制驗證；所有教材程式 CI 改由獨立容器執行。基準有意變更須明確審查。
