# A08｜Rust 推理練習題庫與解答

使用方式：先自己判斷能否編譯、會輸出什麼、為何；再看答案。每題都要求能回到型別、所有權、生命週期或資源管理的根本。

## 題1：重綁定 vs 可變

```rust fragment
let x = 1;
let x = x + 2;
println!("{x}");
```

答案：3。第二個 let 是建立新的同名綁定，不是修改原本的 immutable x。

## 題2：Copy

```rust fragment
let a = 4;
let b = a;
println!("{a} {b}");
```

答案：4 4。整數型別通常為 Copy，a 並未被禁止使用。

## 題3：Move

```rust error E0382
let a = String::from("x");
let b = a;
println!("{a}");
```

答案：不能編譯；String 不實作 Copy，a 的所有權已移給 b。

## 題4：只讀借用

```rust fragment
fn count(s: &str) -> usize { s.len() }
let s = String::from("Hi");
println!("{} {}", count(&s), s);
```

答案：可以；只借用了 s，沒有取得它的所有權。len 是 UTF-8 位元組數。

## 題5：借用衝突

```rust error E0502
let mut xs = vec![1];
let a = &xs[0];
xs.push(2);
println!("{a}");
```

答案：無法編譯；push 可能重新配置，使 a 失效。

## 題6：非詞法生命週期

```rust fragment
let mut s = String::from("A");
let r = &s;
println!("{r}");
s.push('B');
```

答案：可以編譯。r 的最後使用在 push 之前，借用有效使用期不重疊。

## 題7：分號

```rust error E0308
fn f() -> i32 { 8; }
```

答案：無法編譯，函式本體最後結果是 ()。去掉分號即可讓 8 成為結果。

## 題8：Option 不是數字

```rust fragment
let n: Option<i32> = None;
// let result = n + 1;
```

答案：被註解的運算不能直接做；必須先決定 None 時怎麼處理，例如 n.unwrap_or(0) + 1，但這個預設值是否合理要看需求。

## 題9：Result 與錯誤傳播

```rust fragment
fn parse(s: &str) -> Result<i32, std::num::ParseIntError> {
    let n = s.parse::<i32>()?;
    Ok(n * 2)
}
```

答案：合法；parse 失敗會提前回傳 Err，不會自動 panic。n*2 的溢位仍需評估輸入與建構組態。

## 題10：UTF-8

```rust fragment
let text = "中";
println!("{}", text.len());
```

答案：3，len 回傳 UTF-8 位元組長度，不是人眼的字數。

## 題11：Iterator 惰性

```rust fragment
let xs = vec![1, 2, 3];
let mapped = xs.iter().map(|n| n * 2);
let out: Vec<_> = mapped.collect();
```

答案：collect 消費迭代器後 out 為 [2,4,6]。建立 map adapter 本身通常不遍歷所有元素。

## 題12：Rc 與 String.clone

```rust fragment
let a = std::rc::Rc::new(String::from("A"));
let b = std::rc::Rc::clone(&a);
```

答案：a、b 共享同一份 String 的所有權，不是複製出兩份字串。Rc 不可當作跨執行緒 Arc 來使用。

## 題13：Mutex 的保證

四個執行緒各拿 Mutex guard 將共用計數器加一，就能保證所有業務邏輯正確嗎？

答案：不能；Mutex 能保護臨界區的同步存取，不能避免錯誤加總規則、死鎖或遺漏任務。

## 題14：Async 的直覺陷阱

```rust program
async fn job() { println!("hi"); }
fn main() { let _f = job(); }
```

答案：通常不會印出 hi，因建立 future 不等於由 executor poll 並執行到完成。

## 題15：巨集 vs 函式

println! 為何有驚嘆號？

答案：它是巨集呼叫，支援編譯階段處理變長格式引數；不能當成一般 fn 呼叫。

## 題16：型別與業務

bool is_ready + bool is_done 有幾種可能組合？真的需要四種嗎？

答案：兩個 bool 共有四種組合；若業務只允許 Pending、Ready、Done 三種狀態，enum 可排除不合法組合。

## 題17：? 的方向

在回傳 Result 的函式中使用 expr?，如果 expr 是 Err，會繼續執行下面的行嗎？

答案：不會；失敗值會提前從函式傳播（必要時做型別轉換）。

## 題18：編譯器會不會幫你修正演算法？

Rust 可防止很多記憶體安全錯誤，但計算商品折扣時價格捨入規則錯誤會被擋嗎？

答案：不會；必須透過明確規格、測試與業務驗收。

## 題19：只有型別別名是否夠？

type UserId = u64; type OrderId = u64; 會防止誤傳嗎？

答案：不會，兩者只是同一型別的別名。需要防混用時用 struct UserId(u64)、struct OrderId(u64)。

## 題20：何時用 unsafe？

只因借用編譯錯誤就改 raw pointer 是否正確？

答案：通常不正確。先分析所有權與存活需求；unsafe 不會免除記憶體有效、別名和資料競爭等安全義務。

## 最後的解題標準

每題至少說三件事：資料型別是什麼、所有權或生命週期如何流動、若出錯違反了哪一個約束。只有答「能／不能」不算真正掌握。

延伸練習：在 Cargo 專案逐題測試，對錯誤碼使用 rustc --explain 查根因，最後能不看書獨立寫出修正版本。
