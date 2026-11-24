# 架構與發布

## 靜態網站

Node.js 24 使用 markdown-it 解析 CommonMark，highlight.js 在建置時處理 Rust、Shell 和 TOML。每章輸出各自的 `index.html`，網址可分享、深層連結可重新整理。少量瀏覽器原生 JS 提供搜尋、測驗、複製、主題、互動圖解與本機進度。搜尋索引在有搜尋詞時才下載，包含各章文字、程式碼及標題，支援繁體中文子字串與 Rust 診斷碼。閱讀、章節導覽與展開測驗解析可在關閉 JavaScript 時使用。

CommonMark 可以直接在 GitHub 與 HackMD 編輯。技術圖解使用同庫 SVG 和 HTML/CSS，所有權圖由步驟資料驅動。GitHub Pages 提供靜態部署，讀者無須建立帳號，網站無需維護後端和資料庫。`BASE_PATH` 集中處理專案網站子路徑。

課程內容與作者保留於 `content/`。章節名稱、網址、來源與遷移狀態在 `content/catalog.json`。摘要、搜尋索引和頁面均由同一套資料建置。使用者進度留存在同一網域與瀏覽器的 localStorage。清除網站資料或更換裝置會失去記錄。

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

markdown-it 和 highlight.js 在 Node 建置期使用，由 npm 鎖檔固定版本。網站輸出包含兩者的授權文字。Playwright、axe 和 parse5 用於驗證。Rand 0.9.5（MIT 或 Apache-2.0）和 Tokio 1.54.1（MIT）僅用於編譯和執行教材範例，依賴固定在 Cargo.lock。原始教材的公開與授權條款由擁有者決定。 [Rand](https://crates.io/crates/rand/0.9.5) · [Tokio](https://crates.io/crates/tokio/1.54.1)

## 參考專案

- clinic 的權威依範圍定位，這裡採單一教材目錄與來源索引。 [GOVERNANCE.md](https://github.com/waydefu/clinic/blob/main/GOVERNANCE.md)
- world-unbound 的分支隔離與證據要求，用於多人協作及發布界線。 [AGENTS.md](https://github.com/waydefu/world-unbound/blob/main/AGENTS.md)
- Pixel-Games 的按需讀檔與手機驗收區分，用於主導覽和 QA。 [AGENTS.md](https://github.com/waydefu/Pixel-Games/blob/main/AGENTS.md)
- Agent-Tool 的定向檢查和可重現命令，用於教材範例與瀏覽器測試。 [AGENTS.md](https://github.com/waydefu/Agent-Tool/blob/main/AGENTS.md)
- GPU 將結果綁到驗證材料及裝置條件，這裡以 Git 提交和 Actions 分別記錄建置、部署與線上驗證。 [README.md](https://github.com/waydefu/GPU/blob/main/README.md)
