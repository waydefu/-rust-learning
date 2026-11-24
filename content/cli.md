# 第12章｜從一段演算法到真正的工具

## 一、為什麼要做 CLI？

前面學了函式、錯誤、模組與集合，但還缺「外部世界」。命令列工具（CLI）要接收參數、讀檔、回傳結果與錯誤碼，是檢驗程式能否長期維護的理想起點。

本章建立迷你搜尋工具：指定字詞與檔名，輸出包含字詞的行。

## 二、先分清楚純邏輯與副作用

如果搜尋函式直接讀檔、列印、退出，測試就不得不準備檔案並捕捉終端機輸出。比較好的結構是：核心函式只做搜尋；外層負責參數與 I/O。

```rust project mini_grep:src/lib.rs
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|line| line.contains(query)).collect()
}
```

回傳的每一行 &str 都借用 contents，因此在生命週期中標註 'a；query 只用來比對，不需要和輸出綁在一起。

比喻：查資料的分析員不需要自己去門口收郵件；郵務與分析拆開，兩者都容易測。

## 三、命令列參數與 I/O

在 main.rs（假設 package 名為 mini_grep）：

```rust project mini_grep:src/main.rs
use std::env;
use std::fs;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let _program = args.next();
    let query = args.next().ok_or("缺少查詢字詞")?;
    let path = args.next().ok_or("缺少檔案路徑")?;
    let contents = fs::read_to_string(path)?;
    for line in mini_grep::search(&query, &contents) {
        println!("{line}");
    }
    Ok(())
}
```

std::env::args() 取得命令列參數（含程式名稱），以 UTF-8 String 呈現，遇到非 Unicode OS 參數可能有問題；需要更底層時用 args_os。next() 得 Option<String>，ok_or 把 None 換成可傳播的錯誤。

Box<dyn Error> 是教學上通用的錯誤容器；大型工具應評估更清楚的自訂錯誤與 exit code。

## 四、建立、執行與測試

cargo new mini_grep

新增 src/lib.rs 存放上面的 search 函式；將 main.rs 改成上述內容。執行：

```bash
cargo run -- Rust notes.txt
```

-- 的意義：後面參數交給編譯出來的程式，不再解讀為 Cargo 自己的選項。建立 notes.txt 包含幾行文字來驗證。

## 五、所有權為何在 I/O 程式特別重要？

read_to_string 配置並回傳擁有內容的 String；search 回傳的借用指向這個 String。只要 contents 還活著就能安全使用搜尋結果；若在 contents 被釋放後留住借用，編譯器會阻止。

當程式需長時間保存搜尋結果，應選擇複製成 String、記錄位元組位置或在更高層持有原始資料，而非回傳無來源的 &str。

## 六、輸出與錯誤輸出為何要分？

println! 寫入 stdout，eprintln! 寫入 stderr。管線工作流程可能把 stdout 交給下一個程式，若錯誤訊息混進 stdout 會污染資料。

正常內容輸出 stdout；錯誤診斷輸出 stderr。需要自訂錯誤處理時，可把 main 拆成 run() -> Result<...>，在 main 中匹配錯誤、印到 stderr 並設定非零退出碼。

## 七、案例：大小寫與環境變數

可以新增 ignore_case 設定，讓環境變數或命令列旗標控制查詢。注意 String::to_lowercase 會分配新字串，Unicode 大小寫規則有細節；不是所有語言都能靠 ASCII 邏輯處理。

## 八、避免把玩具專案誤認為產品

此版本不支援二進位檔、大檔串流、正規表示式、萬國語言語意匹配或複雜命令列解析。真實專案可用 BufRead 逐行處理大型檔案、clap 解析參數、正規表示式庫做搜尋，但要先了解基本 I/O。

## 九、驗收

練習1：新增 --ignore-case，並為搜尋核心做測試。

練習2：檔案不存在時不可直接 unwrap()，改回傳清楚錯誤。

練習3：改用 BufReader 逐行讀，思考為何此時不能直接回傳借用所有行的 Vec<&str>（每次讀取的緩衝區會被重複利用）。

驗收：能畫出 args → 解析 → 讀檔 → 搜尋純函式 → stdout / stderr 流程。

參考：<https://doc.rust-lang.org/stable/book/ch12-00-an-io-project.html>
