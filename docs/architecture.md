# 架構與發布

## 靜態網站

建置首先呼叫 Rust 2024 Edition 的原生教材檢查工具 `tools/lesson-check`，使用標準函式庫驗證方塊格式及自然標題。接著 Rust 的 pulldown-cmark 產生正文，與 Node.js markdown-it 的既有 DOM 比對以確認相容；Rust 產生 Rust、Shell、TOML 與文字程式區塊的高亮 HTML；highlight.js 只作舊版逐字比對，不相符即拒絕建置。每章輸出各自的 `index.html`，網址可分享、深層連結可重新整理。少量瀏覽器原生 JS 提供搜尋、測驗、複製、主題、互動圖解與本機進度。搜尋索引在有搜尋詞時才下載，包含各章文字、程式碼及標題，支援繁體中文子字串與 Rust 診斷碼。閱讀、章節導覽與展開測驗解析可在關閉 JavaScript 時使用。

CommonMark 可以直接在 GitHub 與 HackMD 編輯。技術圖解使用同庫 SVG 和 HTML/CSS，所有權圖由步驟資料驅動。GitHub Pages 提供靜態部署，讀者無須建立帳號，網站無需維護後端和資料庫。`BASE_PATH` 集中處理專案網站子路徑。

課程內容與作者保留於 `content/`。章節名稱、網址、來源與遷移狀態在 `content/catalog.json`。摘要、搜尋索引和頁面均由同一套資料建置。使用者進度留存在同一網域與瀏覽器的 localStorage。清除網站資料或更換裝置會失去記錄。

## Rust 化的演進方向

正式建置與檢查入口為 `site-render build/check/check-rust/dev`。Rust 使用 serde_json 讀取 catalog 與互動資料、pulldown-cmark 解析 Markdown、scraper/html5ever 解析 HTML5、url 進行 WHATWG URL 解析，並以原生模組產生頁面、搜尋索引、診斷報告及執行教材範例。Node 舊版只在 CI 內做比較與瀏覽器測試。維持既有網址、錨點、HackMD、學習紀錄及純靜態 Pages，保留必要的瀏覽器 JavaScript，不引入 Wasm。

### Rust 正文與 HTML 模板

`tools/site-render` 使用鎖定版 pulldown-cmark 0.13.0（Rust 1.71.1 以上可編譯）解析 CommonMark，開啟表格與刪除線。Rust 接收原始 Markdown、現有章節錨點及高亮程式片段，檢查程式區塊內容與順序、標題層級，轉換教學方塊，拒絕不安全的 URL 並跳脫原始 HTML。Node 端以解析後 DOM 的標籤、屬性與文字（程式碼精確比對）作為相容性閘門；任何正文或高亮差異均停止建置，成功報告記錄零回退。每個 HTML 文件的外層模板均由 Rust 產生，與原模板逐字比對，失敗則停止建置。

舊版 Node 渲染器只用於 CI 的相容性測試，不會被 Rust CLI 呼叫。獨立 job 阻擋 Node/npm/npx 並確認沒有 node_modules。Pages build job 同樣不安裝 npm 依賴。完整網站樹逐位元組比對，渲染報告則比較 JSON 資料；不將報告當作所有未來輸入相容的保證。

### 靜態產物寫入與行為重播

Rust `site-write` 接收所有生成檔案的 UTF-8 欄位，驗證路徑與重複項，原生複製靜態資源，先在暫存目錄完成輸出，再替換 dist；失敗時保留前版（若回復也失敗，保留備份並回報）。拒絕路徑遍歷及資源符號連結。Node 原寫入器僅供 CI 對照。rust-anything 的外部行為錄製與重播在獨立暫存目錄執行，並保留證據；全量建置額外逐位元組比對網站輸出。詳見 [Rust 移植紀錄](rust-port.md)。

### 搜尋索引與預覽介面

`tools/site-tools` 是無第三方依賴的 Rust 2024 crate。原生 Markdown 解析器透過 stdin 傳遞版本化的 UTF-8 欄位（NUL 分隔；建置器拒絕欄位內的 NUL，資料上限 16 MiB），Rust 驗證資料並產生既有 JSON schema，保留標題、診斷碼、程式內容、網址及二至三級錨點。這是兩個 Rust 工具之間的內部介面。原生 check 重新讀取教材並比對搜尋索引完整欄位，CI 再與舊版 JS 索引逐位元組比對。

`npm run dev` 建置後啟動 Rust `site-preview`，支援 GET/HEAD、子路徑、目錄索引、308、404、MIME 類型與簡單的 HTTP 請求大小及讀寫逾時限制。它只供本機開發，預設綁定 loopback；會解碼並檢查路徑、拒絕遍歷及指向根目錄外的符號連結，但不是完整生產級 HTTP 實作。GitHub Pages 仍直接發布 dist，預覽工具不會在公開網站上執行。CI 會用 curl 實際請求預覽伺服器並比對搜尋索引。

## HackMD 的編輯方式

1. 維護者在 GitHub 安裝 HackMD GitHub App，授權本儲存庫。首次設定先用一個編輯分支和一份 Markdown 試走流程。
2. 從最新 `main` 建立分支，例如 `content/ownership`。在 HackMD 的「版本與 GitHub 同步」連結該分支的 `content/ownership.md`，用 **Pull** 取得最新 Markdown。
3. 保持 CommonMark 標題與程式碼柵欄。建議用 HackMD 的 CommonMark 換行規則。編輯完成後儲存命名版本，在同一面板手動 **Push** 到該分支。每次編輯會儲存在 HackMD，Push 才會建立 GitHub 提交。
4. 在 GitHub 審查差異並建立 PR。碰到同章重疊修改時，先在 GitHub 整合最新版本，再 Pull 回 HackMD，逐段核對衝突。在 PR 合併後以 `main` 作為下一輪編輯的起點。
5. PR 通過檢查並審查合併後，GitHub Pages 工作流程自動建置並部署。已公開的內容可透過 revert 提交回退，再由工作流程重新部署。

HackMD 同步教學也列出免費方案的同步次數限制。實際配額以帳戶當時的方案為準。可直接在 GitHub 或本機編輯同一個 Markdown，流程後段相同。Google Drive 保留作歷史來源，變更在 GitHub 審查。若使用 HackMD App，確認它有目標分支的寫入權限，並保留 `main` 的人工審查。 [HackMD Sync a Note with GitHub](https://hackmd.io/s/link-with-github)

## 發布與驗證

維護者確認內容的公開授權後，在 GitHub 的 **Settings → Pages → Source** 選擇 **GitHub Actions**。工作流程由 PR 驗證或由 `main` 發布，也可在 Actions 手動啟動。Pages 部署權限僅在部署 job 使用。部署成功之後，同一工作流程會請求首頁、第 4 章深層網址與共用 CSS。應以 Actions 實際產生的網址發布。 [GitHub Pages 發布來源](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site) · [Pages 官方 Actions](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)

變更的證據放在 PR 和 Actions。實作指的是可追溯的差異。已測試需有執行結果。已部署需有部署紀錄。公開可用需由部署後請求確認。真機、各瀏覽器和使用者學習成果依各自的驗收範圍記錄。

## 來源與依賴

pulldown-cmark 及其建置期依賴以 tools/site-render/Cargo.lock 鎖定。markdown-it 與 highlight.js 在 Node 建置期提供舊版相容性對照，由 npm 鎖檔固定版本。網站輸出包含兩者的授權文字。Playwright、axe 和 parse5 用於驗證。Rand 0.9.5（MIT 或 Apache-2.0）和 Tokio 1.54.1（MIT）僅用於編譯和執行教材範例，依賴固定在 Cargo.lock。原始教材的公開與授權條款由擁有者決定。 [Rand](https://crates.io/crates/rand/0.9.5) · [Tokio](https://crates.io/crates/tokio/1.54.1)

## 參考專案

- clinic 的權威依範圍定位，這裡採單一教材目錄與來源索引。 [GOVERNANCE.md](https://github.com/waydefu/clinic/blob/main/GOVERNANCE.md)
- world-unbound 的分支隔離與證據要求，用於多人協作及發布界線。 [AGENTS.md](https://github.com/waydefu/world-unbound/blob/main/AGENTS.md)
- Pixel-Games 的按需讀檔與手機驗收區分，用於主導覽和 QA。 [AGENTS.md](https://github.com/waydefu/Pixel-Games/blob/main/AGENTS.md)
- Agent-Tool 的定向檢查和可重現命令，用於教材範例與瀏覽器測試。 [AGENTS.md](https://github.com/waydefu/Agent-Tool/blob/main/AGENTS.md)
- GPU 將結果綁到驗證材料及裝置條件，這裡以 Git 提交和 Actions 分別記錄建置、部署與線上驗證。 [README.md](https://github.com/waydefu/GPU/blob/main/README.md)

### 動效與無障礙

首頁啟動提示使用 CSS 與既有瀏覽器 JS，在每個分頁工作階段至多播放一次；無遮罩、不阻擋點擊、不搶焦點，深層錨點與 reduced-motion 不播放。卡片、按鈕、閱讀頁首、對話框、測驗及記憶體圖採短時動畫，不延遲導覽或正文。無 JavaScript 時教材仍直接可讀。Playwright 驗證啟動提示、工作階段與減少動態效果。
