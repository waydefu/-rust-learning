# 架構與發布

## 靜態網站

建置首先呼叫 Rust 2024 Edition 的原生教材檢查工具 `tools/lesson-check`，使用標準函式庫驗證方塊格式及自然標題。接著 Rust 的 pulldown-cmark 產生正文，與 Node.js markdown-it 的既有 DOM 比對以確認相容；highlight.js 仍在建置時處理 Rust、Shell 和 TOML，提供經過跳脫的程式區塊。每章輸出各自的 `index.html`，網址可分享、深層連結可重新整理。少量瀏覽器原生 JS 提供搜尋、測驗、複製、主題、互動圖解與本機進度。搜尋索引在有搜尋詞時才下載，包含各章文字、程式碼及標題，支援繁體中文子字串與 Rust 診斷碼。閱讀、章節導覽與展開測驗解析可在關閉 JavaScript 時使用。

CommonMark 可以直接在 GitHub 與 HackMD 編輯。技術圖解使用同庫 SVG 和 HTML/CSS，所有權圖由步驟資料驅動。GitHub Pages 提供靜態部署，讀者無須建立帳號，網站無需維護後端和資料庫。`BASE_PATH` 集中處理專案網站子路徑。

課程內容與作者保留於 `content/`。章節名稱、網址、來源與遷移狀態在 `content/catalog.json`。摘要、搜尋索引和頁面均由同一套資料建置。使用者進度留存在同一網域與瀏覽器的 localStorage。清除網站資料或更換裝置會失去記錄。

## Rust 化的演進方向

目前是混合式建置：Rust 負責教材預檢、搜尋索引、本機 HTTP 預覽、CommonMark 正文候選與共用 HTML 文件模板；Rust 亦從 Markdown 產生章節標題、Unicode 錨點與共用側欄導覽，逐項或逐字與舊版比對。Node.js 仍負責 catalog JSON 讀取、搜尋全文、語法高亮、部分頁面內容元件、舊渲染器及相容性比對。後續可移植章節資料、導覽、互動元件的靜態模板、語法高亮及教材範例抽取；只有在相容性報告確認後才能移除舊工具。GitHub Pages 只接受靜態輸出，因此不需要改成持續運作的 Rust HTTP 伺服器。瀏覽器中的互動仍使用少量 JavaScript，只有適合的計算模組才評估 Rust/Wasm。維持既有網址、錨點、HackMD 編輯方式及瀏覽器學習紀錄。尚未完成全站 Rust 移植。

### Rust 正文與 HTML 模板

`tools/site-render` 使用鎖定版 pulldown-cmark 0.13.0（Rust 1.71.1 以上可編譯）解析 CommonMark，開啟表格與刪除線。Rust 接收原始 Markdown、現有章節錨點及高亮程式片段，檢查程式區塊內容與順序、標題層級，轉換教學方塊，拒絕不安全的 URL 並跳脫原始 HTML。Node 端以解析後 DOM 的標籤、屬性與文字（程式碼精確比對）作為相容性閘門，只有通過才替換正文；每篇回退都有可見報告，`REQUIRE_NATIVE_MARKDOWN=1` 可作為完整遷移的嚴格驗收。每個 HTML 文件的外層模板均由 Rust 產生，與原模板逐字比對，失敗則停止建置。

目前仍需 Node.js，並且兩套渲染器在建置期有額外成本；網站送到瀏覽器的資源量不因這項過渡設計增加（另有獨立的建置報告，不會由一般閱讀流程下載）。沒有實測證據前，不宣稱全站已純 Rust 化或效能提升。

### 靜態產物寫入與行為重播

Rust `site-write` 接收所有生成檔案的 UTF-8 欄位，驗證路徑與重複項，原生複製靜態資源，先在暫存目錄完成輸出，再替換 dist；失敗時保留前版（若回復也失敗，保留備份並回報）。拒絕路徑遍歷及資源符號連結。Node 原寫入器僅供 CI 對照。rust-anything 的外部行為錄製與重播在獨立暫存目錄執行，並保留證據；全量建置額外逐位元組比對網站輸出。詳見 [Rust 移植紀錄](rust-port.md)。

### 搜尋索引與預覽介面

`tools/site-tools` 是無第三方依賴的 Rust 2024 crate。現有 Markdown 解析器透過 stdin 傳遞版本化的 UTF-8 欄位（NUL 分隔；建置器拒絕欄位內的 NUL，資料上限 16 MiB），Rust 驗證資料並產生既有 JSON schema，保留標題、診斷碼、程式內容、網址及二至三級錨點。這是過渡介面，未來 Rust Markdown 解析器可直接建立索引資料結構，不必傳輸中間格式。`npm run check` 逐欄比對原先的 JS 索引契約，避免影響搜尋。

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

pulldown-cmark 及其建置期依賴以 tools/site-render/Cargo.lock 鎖定。markdown-it 和 highlight.js 在 Node 建置期使用，由 npm 鎖檔固定版本。網站輸出包含兩者的授權文字。Playwright、axe 和 parse5 用於驗證。Rand 0.9.5（MIT 或 Apache-2.0）和 Tokio 1.54.1（MIT）僅用於編譯和執行教材範例，依賴固定在 Cargo.lock。原始教材的公開與授權條款由擁有者決定。 [Rand](https://crates.io/crates/rand/0.9.5) · [Tokio](https://crates.io/crates/tokio/1.54.1)

## 參考專案

- clinic 的權威依範圍定位，這裡採單一教材目錄與來源索引。 [GOVERNANCE.md](https://github.com/waydefu/clinic/blob/main/GOVERNANCE.md)
- world-unbound 的分支隔離與證據要求，用於多人協作及發布界線。 [AGENTS.md](https://github.com/waydefu/world-unbound/blob/main/AGENTS.md)
- Pixel-Games 的按需讀檔與手機驗收區分，用於主導覽和 QA。 [AGENTS.md](https://github.com/waydefu/Pixel-Games/blob/main/AGENTS.md)
- Agent-Tool 的定向檢查和可重現命令，用於教材範例與瀏覽器測試。 [AGENTS.md](https://github.com/waydefu/Agent-Tool/blob/main/AGENTS.md)
- GPU 將結果綁到驗證材料及裝置條件，這裡以 Git 提交和 Actions 分別記錄建置、部署與線上驗證。 [README.md](https://github.com/waydefu/GPU/blob/main/README.md)
