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
| Markdown 正文 | 30/31 已驗證；剩餘差異已提交修正 | CI 強制 31/31，禁止回退 |
| 靜態產物寫入 | Rust 實作已提交，待 CI | rust-anything record/replay 與完整產物逐位元組對照 |
| 章節元資料、Unicode 錨點 | 未移植 | 固定現有資料與 Unicode 邊界案例 |
| 導覽、首頁、互動教材模板 | 未移植 | 34 頁 DOM 與瀏覽器測試 |
| 語法高亮、範例抽取、網站檢查 | 未移植 | fenced code、診斷、測試與輸出契約 |
| Rust CLI 總入口 | 未移植 | 完整 build/check/dev 行為錄製與重播 |

## 第一個 rust-anything 單元：site-write

`scripts/migration/legacy-site-write.mjs` 將原版 build.mjs 的 rm/mkdir/cp/writeFile 序列移入測試專用 adapter；CLI 封包及輸入拒絕規則為新加入的明確契約，不是聲稱舊內部函式已支援任意非法輸入。Rust 新寫入器先建立完整暫存樹，再替換舊 dist，並另外驗證二進位資源、符號連結和失敗保留舊產物。

`write-cases.mjs` 產生 14 個明確案例（含繁體中文、Emoji、空檔、巢狀路徑、舊檔移除、重複及路徑遍歷）。`replay-site-write.sh` 在隔離目錄執行上游工具：

1. app record 以 Node 參考程式執行每案兩次，不接受不穩定、噪音或逾時案例。
2. app replay 重播原版，全部必須通過。
3. app replay --build ./site-write 重播 Rust，全部必須通過，不允許跳過或回退。
4. 原始 cases、JSON 結果與差異報告作為 CI artifact，不發布到網站。
5. VERIFY_SITE_WRITE=1 對根路徑與 Pages 子路徑生成的完整網站樹逐檔比對原版 writer。不是整個建置器已純 Rust 的證據。

本輪不執行 app lift：它需要完整入口與所有單元通過，尚未達成。不得將測試尚未執行、或僅有移植程式碼提交，記為已完成驗收。
