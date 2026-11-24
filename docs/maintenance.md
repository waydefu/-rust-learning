# 維護與驗證收尾

## 儲存庫管理（需管理權限）

2026-11-24 讀取 GitHub main protected=false、rulesets 為空；PR #15 仍開啟。工具無 branch delete/ruleset write 接口，因此下列為待維護者操作，不宣稱已改設定。

在 Settings → Rules → Rulesets 建立 main 規則：Require a pull request before merging、Require status checks to pass、Block force pushes、Restrict deletions。待 PR #15 的新工作全部出現並驗證後，選擇六個驗收 job（教材與網站驗證、無 Node 的 Rust CLI 驗證、跨瀏覽器與 Android 模擬、Rust 品質與依賴稽核、隔離環境教材驗證、Rust 測試覆蓋率）；不把僅 main 執行的部署 job 設為 PR 必要檢查。按團隊情況決定審核人數，避免單人專案把自己鎖住。可開啟合併後自動刪除 head branch。

下列 14 個分支的當前 tip 與已合併 PR 的 head SHA 完全相符，僅在確認未被重新使用且無其他依賴時刪除；不刪 main、目前 PR #15 分支。GitHub Branches 頁面可逐一確認 merged 狀態。新產物基準已入庫，不需為舊版 oracle 保留分支。

| PR | 分支 |
| --- | --- |
| 1 | fix/ci-rust-toolchain |
| 2 | fix/pages-smoke-portability |
| 3 | content/zero-basics-ownership |
| 4 | feat/course-reading-callouts |
| 5 | fix/pages-artifact-visibility |
| 6 | feat/natural-lessons-rust-tooling |
| 7 | feat/rust-search-preview |
| 8 | feat/rust-native-render |
| 9 | feat/rust-anything-replay |
| 10 | feat/rust-metadata-navigation |
| 11 | fix/pages-artifact-job-boundary |
| 12 | feat/rust-home-cards |
| 13 | feat/native-cli-motion |
| 14 | fix/motion-quality-hardening |

## 真機驗收

CI 的 Pixel 7 模擬不是物理裝置。維護者在實際 Android 裝置以 Chrome 及行動網路，驗證首頁、章節、目錄、搜尋、程式碼橫向捲動、互動實驗、測驗、深連結、系統減少動態效果。使用 Chrome DevTools 遠端偵錯擷取三次冷載入 Performance trace；記錄裝置、OS、瀏覽器版本、網路與部署 SHA，並分別記錄 FCP/LCP/CLS、操作延遲。不可將模擬結果標示成真機。

## 覆蓋率

Actions 的 rust-coverage 量測 Rust 單元測試與原生 build/check，僅為已觀察執行範圍；隔離範例、HTTP 預覽、瀏覽器與前端覆蓋率不在同一統計母體。先以第一份有效報告建立基準，再按實際未覆蓋分支補測；不任意填寫百分比或以任意高門檻製造假保障。
