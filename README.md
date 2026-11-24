# Rust 學習工坊

繁體中文 Rust 2024 Edition 課程。從工具鏈和第一個程式出發，涵蓋所有權、借用、生命週期、智慧指標、並行、非同步和本機 HTTP 伺服器。包含 21 章、8 份延伸教材、互動記憶體圖和各章自我測驗。

## 本機使用

建置網站需要 Node.js 24、Cargo 及 Rust 1.85.1（由 `rust-toolchain.toml` 指定）。Rust 原生工具驗證教材、產生章節標題與錨點、搜尋索引、側欄導覽、渲染 Markdown 與共用 HTML 頁面模板、寫入網站產物，並提供本機預覽。為了維持既有網站相容性，Node.js 仍提供舊渲染對照、語法高亮、部分頁面內容與互動元件。一般閱讀只需瀏覽器。

```sh
npm ci
npm run dev
```

開啟終端機顯示的本機網址（預設 127.0.0.1:4173）。預覽由 Rust 的 site-preview 提供；修改教材後重新啟動。若需在可信任區域網路上測試手機，可明確設定 HOST=0.0.0.0，勿暴露公網。站內進度和最後閱讀位置儲存在目前瀏覽器及網站來源。

```sh
npm run build
npm run check
npm run check:rust
cargo test --locked --manifest-path tools/lesson-check/Cargo.toml
cargo test --locked --manifest-path tools/site-tools/Cargo.toml
cargo test --locked --manifest-path tools/site-render/Cargo.toml
BASE_PATH=/-rust-learning/ npm run build
BASE_PATH=/-rust-learning/ npm run check
npx playwright install chromium
npm run test:browser
```

`dist/` 是靜態輸出。以本機或其他網域根目錄提供時用預設 `BASE_PATH=/`。驗證 GitHub 專案路徑時可設 `BASE_PATH=/-rust-learning/`。GitHub Actions 會讀取 Pages 的實際子路徑。Playwright 的預設測試攔截測試網域的請求，從 Pages 產物回應。也可設定 `TEST_ORIGIN` 指向已啟動的伺服器，該伺服器需用同一 Pages 子路徑。

## 渲染相容性

每次建置會將 Rust 正文與原本 Markdown 渲染的 DOM 比對。相容時使用 Rust 輸出，否則在終端與 `dist/build-render-report.json` 記錄原因並保留該篇舊正文；共用 HTML 文件模板一律由 Rust 生成，與原模板逐字比對。若要禁止正文回退，可設定 `REQUIRE_NATIVE_MARKDOWN=1` 執行建置。此過渡機制不是完全移除 Node.js 的宣告。

## Rust 移植驗證

建置器使用 Rust 的 `site-write` 建立網站輸出、複製資源並替換 `dist`；Node 原版寫入器保留在 `scripts/migration/`，只作測試對照。CI 會逐位元組比較完整網站輸出，並使用固定版本的 rust-anything v4.0.0 對獨立寫入器的成功與錯誤案例執行錄製及重播。Linux x86_64 可執行 `bash scripts/migration/replay-site-write.sh`；它只在暫存目錄下載經 SHA-256 驗證的工具，不變更 Agent 設定或教材工具鏈。詳見 [移植紀錄](docs/rust-port.md)。

正式 CI 與 Pages 建置使用 `REQUIRE_NATIVE_MARKDOWN=1`，任何正文回退都會阻止發布；本機仍可用既有相容性報告診斷差異。

## 專案位置

| 位置 | 用途 |
| --- | --- |
| [content/](content/) | 教材 Markdown、章節路徑與來源索引 |
| [src/](src/) | 頁面模板、圖解、測驗、樣式和瀏覽器互動 |
| [scripts/](scripts/) | 產生靜態頁面、檢查內容與 Rust 範例 |
| [tools/lesson-check/](tools/lesson-check/) | Rust 原生教材建置前檢查器 |
| [tools/site-tools/](tools/site-tools/) | Rust 原生搜尋索引、網站產物寫入與本機 HTTP 預覽 |
| [tools/site-render/](tools/site-render/) | Rust CommonMark 渲染與 HTML 文件模板 |
| [examples/](examples/) | 外部 crate 範例的 Cargo 設定與鎖檔。程式碼從教材抽取 |
| [tests/](tests/) | Playwright 操作與 axe 無障礙檢查 |
| [.github/workflows/site.yml](.github/workflows/site.yml) | Pull request 驗證與 Pages 部署 |
| [docs/architecture.md](docs/architecture.md) | 架構、HackMD 協作、部署與回退 |
| [docs/migration.md](docs/migration.md) | 教材來源、實際修改與逐章驗收 |
| [AGENTS.md](AGENTS.md) | 多人與 Agent 協作入口 |

## 目標儲存庫

GitHub 目前可查到名稱含前導連字號的 [waydefu/-rust-learning](https://github.com/waydefu/-rust-learning)。公開推送、輸出版權與 GitHub Pages 設定由維護者確認。上線狀態以 Actions 部署結果與實際 HTTP 驗證為準。
