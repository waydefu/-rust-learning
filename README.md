# Rust 學習工坊

繁體中文 Rust 2024 Edition 課程。從工具鏈和第一個程式出發，涵蓋所有權、借用、生命週期、智慧指標、並行、非同步和本機 HTTP 伺服器。包含 21 章、8 份延伸教材、互動記憶體圖和各章自我測驗。

## 本機使用

建置網站只需要 Cargo 及 Rust 1.85.1（由 rust-toolchain.toml 指定）。正式建置、檢查、教材範例與本機預覽均由 Rust CLI 提供，不需要 Node.js、npm 或 node_modules。Node.js 24 只用於測試。舊版 Node 建置器與遷移協定均已退役，原生測試使用已核准的固定產物雜湊。一般閱讀只需瀏覽器。

```sh
cargo run --locked --manifest-path tools/site-render/Cargo.toml -- dev
```

開啟終端機顯示的本機網址（預設 127.0.0.1:4173）。預覽由 Rust 的 site-preview 提供；修改教材後重新啟動。若需在可信任區域網路上測試手機，可明確設定 HOST=0.0.0.0，勿暴露公網。站內進度和最後閱讀位置儲存在目前瀏覽器及網站來源。

```sh
cargo run --locked --manifest-path tools/site-render/Cargo.toml -- build
cargo run --locked --manifest-path tools/site-render/Cargo.toml -- check
cargo run --locked --manifest-path tools/site-render/Cargo.toml -- check-rust
cargo test --locked --workspace --manifest-path tools/Cargo.toml
BASE_PATH=/-rust-learning/ cargo run --locked --manifest-path tools/site-render/Cargo.toml -- build
BASE_PATH=/-rust-learning/ cargo run --locked --manifest-path tools/site-render/Cargo.toml -- check
# 僅做測試時需要 Node：
npm ci
npx playwright install chromium
npm run test:browser
```

`dist/` 是靜態輸出。以本機或其他網域根目錄提供時用預設 `BASE_PATH=/`。驗證 GitHub 專案路徑時可設 `BASE_PATH=/-rust-learning/`。GitHub Actions 會讀取 Pages 的實際子路徑。Playwright 的預設測試攔截測試網域的請求，從 Pages 產物回應。也可設定 `TEST_ORIGIN` 指向已啟動的伺服器，該伺服器需用同一 Pages 子路徑。

## 原生渲染與驗收

正式 CLI 以具名 Rust 型別讀取 catalog、site-data.json、Markdown、搜尋索引與渲染報告，直接呼叫教材檢查、搜尋索引、網站寫入及 HTTP 預覽函式庫。三個工具共享 Cargo Workspace 與鎖檔；正式路徑不含 Node、歷史 NUL 協定或子 Cargo 建置器。教材外部依賴專案仍獨立。

CI 驗證根路徑與 Pages 子路徑完整網站樹的逐檔 SHA-256，基準來自已完成舊版對照與 rust-anything 重播的成功 CI，而非目前建置結果。另驗證建置可重現、拒絕無效路徑/內容/JSON schema、網站檢查負例及失敗保留舊產物。基準來源及有意更新方式見 [tests/fixtures/README.md](tests/fixtures/README.md)，歷史遷移證據見 [移植紀錄](docs/rust-port.md)。不再下載或執行歷史 Node 實作。

`dist/build-render-report.json` 記錄原生正文及高亮統計，無 Node 回退。無 Node 的獨立 CI 驗證建置和檢查，所有可執行教材另在隔離容器驗證。

## 專案位置

| 位置 | 用途 |
| --- | --- |
| [content/](content/) | 教材 Markdown、章節路徑與來源索引 |
| [src/](src/) | 樣式和瀏覽器互動 |
| [content/site-data.json](content/site-data.json) | 共用課程階段、SVG 圖示、互動圖解與測驗資料 |
| [scripts/](scripts/) | 原生回歸與容器隔離驗收 |
| [tools/lesson-check/](tools/lesson-check/) | Rust 原生教材建置前檢查器 |
| [tools/site-tools/](tools/site-tools/) | Rust 原生搜尋索引、網站產物寫入與本機 HTTP 預覽 |
| [tools/site-render/](tools/site-render/) | Rust CommonMark、高亮、全站模板、互動資料、範例執行器與網站驗證 |
| [examples/](examples/) | 外部 crate 範例的 Cargo 設定與鎖檔。程式碼從教材抽取 |
| [tests/](tests/) | Playwright 操作與 axe 無障礙檢查 |
| [.github/workflows/site.yml](.github/workflows/site.yml) | Pull request 驗證與 Pages 部署 |
| [docs/architecture.md](docs/architecture.md) | 架構、HackMD 協作、部署與回退 |
| [docs/migration.md](docs/migration.md) | 教材來源、實際修改與逐章驗收 |
| [AGENTS.md](AGENTS.md) | 多人與 Agent 協作入口 |

## 目標儲存庫

GitHub 目前可查到名稱含前導連字號的 [waydefu/-rust-learning](https://github.com/waydefu/-rust-learning)。公開推送、輸出版權與 GitHub Pages 設定由維護者確認。上線狀態以 Actions 部署結果與實際 HTTP 驗證為準。


## 品質與互動驗收

- 跨瀏覽器抽測：先用 Pages 子路徑建置，再執行 `npx playwright install chromium firefox webkit` 和 `npx playwright test --config=playwright.cross.config.mjs`。Android 為瀏覽器模擬，不是實機。
- Rust 格式、Clippy 與 RustSec 依賴漏洞檢查是強制 CI 關卡；原有正式建置、回歸、瀏覽器與部署檢查仍須通過。RustSec 的無修補版本維護性警告需定期審查，不等同已知漏洞。
- 集合、迭代器、智慧指標、並行與非同步增加可操作的概念模型；它們不是實際編譯器、記憶體配置器或執行緒排程器。停用 JS 時仍可閱讀教材。
- 具確定性輸出的 `rust run` 可逐行附上 `// 預期輸出: 文字`，原生驗證器會比對完整執行輸出。所有 `rust run` 範例必須明確標註預期 stdout 或 `// 預期無輸出`。

### 前端模組

瀏覽器入口 `src/assets/app.js` 僅負責初始化；`modules/` 分別管理共用資料、導覽搜尋、學習進度、閱讀互動、動效及概念實驗室。`site.css` 保留基礎設計及教材排版，`motion.css` 和 `concept-labs.css` 管理對應功能。所有入口與 modulepreload 由 HTML 宣告，維持原有靜態資源驗證與大小預算。

### 教材執行與隔離

所有 `rust run` 範例須宣告 stdout 契約（`// 預期輸出: 文字` 或 `// 預期無輸出`）。執行器分離 stdout/stderr，限制輸出、逾時及繼承環境變數，Unix 逾時終止程序群組。本機直接 `check-rust` 仍只適用可信任教材。

Linux Docker 環境執行 `bash scripts/regression/check-sandbox.sh`，先取得鎖定的依賴，再在無外網、唯讀根檔案系統、非 root、移除 capabilities、限制 CPU/記憶體/PID、僅有 tmpfs 可寫且無主機目錄掛載的容器中驗證全部教材。CI 將此列為發布關卡；仍信任 Docker 基礎映像與編譯器，不是對敵意程式的絕對安全保證。
