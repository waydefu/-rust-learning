# Rust 學習工坊

繁體中文 Rust 2024 Edition 課程。從工具鏈和第一個程式出發，涵蓋所有權、借用、生命週期、智慧指標、並行、非同步和本機 HTTP 伺服器。包含 21 章、8 份延伸教材、互動記憶體圖和各章自我測驗。

## 本機使用

需要 Node.js 24。編譯教材範例另需 Rustup 和 Rust 1.85.1（由 `rust-toolchain.toml` 指定）。一般閱讀只需瀏覽器。

```sh
npm ci
npm run dev
```

開啟終端機顯示的本機網址。修改教材後重新啟動本機預覽。站內進度和最後閱讀位置儲存在目前瀏覽器及網站來源。

```sh
npm run build
npm run check
npm run check:rust
BASE_PATH=/-rust-learning/ npm run build
BASE_PATH=/-rust-learning/ npm run check
npx playwright install chromium
npm run test:browser
```

`dist/` 是靜態輸出。以本機或其他網域根目錄提供時用預設 `BASE_PATH=/`。驗證 GitHub 專案路徑時可設 `BASE_PATH=/-rust-learning/`。GitHub Actions 會讀取 Pages 的實際子路徑。Playwright 的預設測試攔截測試網域的請求，從 Pages 產物回應。也可設定 `TEST_ORIGIN` 指向已啟動的伺服器，該伺服器需用同一 Pages 子路徑。

## 專案位置

| 位置 | 用途 |
| --- | --- |
| [content/](content/) | 教材 Markdown、章節路徑與來源索引 |
| [src/](src/) | 頁面模板、圖解、測驗、樣式和瀏覽器互動 |
| [scripts/](scripts/) | 產生靜態頁面、檢查內容與 Rust 範例 |
| [examples/](examples/) | 外部 crate 範例的 Cargo 設定與鎖檔。程式碼從教材抽取 |
| [tests/](tests/) | Playwright 操作與 axe 無障礙檢查 |
| [.github/workflows/site.yml](.github/workflows/site.yml) | Pull request 驗證與 Pages 部署 |
| [docs/architecture.md](docs/architecture.md) | 架構、HackMD 協作、部署與回退 |
| [docs/migration.md](docs/migration.md) | 教材來源、實際修改與逐章驗收 |
| [AGENTS.md](AGENTS.md) | 多人與 Agent 協作入口 |

## 目標儲存庫

GitHub 目前可查到名稱含前導連字號的 [waydefu/-rust-learning](https://github.com/waydefu/-rust-learning)。公開推送、輸出版權與 GitHub Pages 設定由維護者確認。上線狀態以 Actions 部署結果與實際 HTTP 驗證為準。
