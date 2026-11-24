# 工作方式

- 從 [README.md](README.md) 定位入口。改教材時讀目標 Markdown、相關的 `content/catalog.json` 項目和 [遷移紀錄](docs/migration.md)，其餘檔案按需查閱。
- GitHub 預設分支的 `content/` 是教材權威來源。Google Drive 留作原始對照，HackMD 用於編輯分支。來源、架構決策與驗證證據各自只維護一次。
- 多位人類或 Agent 同時工作時，用各自的分支與工作目錄，先查同章或共用元件是否已有 PR。PR 描述修改範圍、實際驗證和未驗證項目。協調共用導覽、建置與資料契約的改動再整合。
- 僅修改任務相關檔案。章節內容由 Markdown 維護，課程階段、圖解、測驗與共用圖示維護於 `content/site-data.json`。用 Git 回退內容或程式碼。
- 驗證依變更範圍選擇。在本機執行的指令、Actions 結果、部署結果、線上實測分開記錄。手機裝置測試請標明是實機或瀏覽器模擬。
- 外部服務安裝或設定、正式公開發布、付費、不可逆刪除與機密處理須取得對應授權。

- 舊版 Node 實作已退役，固定歷史參考僅透過 `scripts/regression/verify-reference.sh` 讀取；不要重新引入雙套渲染或建置邏輯。預期輸出契約變更須附明確差異與回歸證據。
