# Rust 學習工坊

繁體中文 Rust 2024 Edition 課程。從工具鏈和第一個程式出發，涵蓋所有權、借用、生命週期、智慧指標、並行、非同步和本機 HTTP 伺服器。包含 21 章、8 份延伸教材、互動記憶體圖和各章自我測驗。

## 本機使用

建置網站只需要 Cargo 及 Rust 1.85.1（由 rust-toolchain.toml 指定）。正式建置、檢查、教材範例與本機預覽均由 Rust CLI 提供，不需要 Node.js、npm 或 node_modules。Node.js 24 只用於 CI 的舊版對照及 Playwright 瀏覽器測試。一般閱讀只需瀏覽器。

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
# 僅做舊版與瀏覽器測試時需要 Node：
npm ci
npx playwright install chromium
npm run test:browser
```

`dist/` 是靜態輸出。以本機或其他網域根目錄提供時用預設 `BASE_PATH=/`。驗證 GitHub 專案路徑時可設 `BASE_PATH=/-rust-learning/`。GitHub Actions 會讀取 Pages 的實際子路徑。Playwright 的預設測試攔截測試網域的請求，從 Pages 產物回應。也可設定 `TEST_ORIGIN` 指向已啟動的伺服器，該伺服器需用同一 Pages 子路徑。

## 渲染相容性

正式 Rust CLI 直接讀取 catalog、site-data.json 和 Markdown，原生生成全部網站產物。CI 使用舊版建置器比對根路徑及 Pages 子路徑的完整網站樹；任何差異均阻止合併與部署。成功的 `dist/build-render-report.json` 記錄各篇正文及各語言原生生成數量，沒有回退到 Node 的路徑。

## Rust 移植驗證

建置器使用 Rust 的 `site-write` 建立網站輸出、複製資源並替換 `dist`；Node 原版寫入器保留在 `scripts/migration/`，只作測試對照。CI 會逐位元組比較完整網站輸出，並使用固定版本的 rust-anything v4.0.0 對獨立寫入器及完整 CLI（build/check/check-rust/dev、失敗保留產物）執行錄製及重播。Linux x86_64 可執行 `bash scripts/migration/replay-site-write.sh`；它只在暫存目錄下載經 SHA-256 驗證的工具，不變更 Agent 設定或教材工具鏈。詳見 [移植紀錄](docs/rust-port.md)。

另有獨立 CI job 在沒有 node_modules 且阻擋 node/npm/npx 的 PATH 下執行 build、check、check-rust，Pages 發布也直接使用 Rust CLI。舊版只保留為測試對照。

## 專案位置

| 位置 | 用途 |
| --- | --- |
| [content/](content/) | 教材 Markdown、章節路徑與來源索引 |
| [src/](src/) | 舊版驗收對照、樣式和瀏覽器互動 |
| [content/site-data.json](content/site-data.json) | 共用課程階段、SVG 圖示、互動圖解與測驗資料 |
| [scripts/](scripts/) | 產生靜態頁面、檢查內容與 Rust 範例 |
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
- 具確定性輸出的 `rust run` 可逐行附上 `// 預期輸出: 文字`，原生驗證器會比對完整執行輸出。未標註的範例仍只驗證結束狀態。

### 前端模組

瀏覽器入口 `src/assets/app.js` 僅負責初始化；`modules/` 分別管理共用資料、導覽搜尋、學習進度、閱讀互動、動效及概念實驗室。`site.css` 保留基礎設計及教材排版，`motion.css` 和 `concept-labs.css` 管理對應功能。所有入口與 modulepreload 由 HTML 宣告，維持原有靜態資源驗證與大小預算。

Rust 工具統一使用 `tools/Cargo.toml` workspace、`tools/Cargo.lock` 與 `tools/target/`；教材的外部依賴範例維持獨立 Cargo 專案，避免教學依賴影響網站工具。
