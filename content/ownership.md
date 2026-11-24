# 第04章｜所有權、借用與切片：Rust 的核心

理解這章後，你才算真正開始懂 Rust。目標是能預測程式何時移動、複製、借用、釋放，而非死記「有些寫法不行」。

## 一、問題從哪裡來：記憶體生命週期

程式在執行時需要保存資料。對大小固定的 i32，通常能直接放入區域值的儲存空間；對長度會改變的文字，String 通常在堆積（heap）配置連續位元組，局部的 String 值包含指向緩衝區的指標、長度與容量等資訊。

堆積配置需要分配、回收；若忘記回收就是洩漏，太早回收就可能留下懸空指標，同一份配置回收兩次更可能造成記憶體破壞。

傳統做法有手動 free 或垃圾回收（GC）。Rust 選擇另一條路：利用編譯器檢查所有權與借用，多數正常資源在離開作用域時自動清理，不依賴常駐的追蹤式 GC。

比喻：堆積是圖書館倉庫；持有所有權者是唯一負責歸還該書的人；借用者有閱覽權，但不自動取得「歸還責任」。要注意這是責任分配模型，不代表任何時刻只有一個人能讀，也不是所有類型都真的放在堆積。

## 二、三條基本規則，因果連續

1. 先以一個直接持有的 `String` 為例，每份配置有一個負責清理的所有權位置。共享所有權是 [第 15 章](../smart-pointers/) 的另一種模型。

2. 所有者離開有效作用域時，值通常會被清理（Drop 可以自訂清理行為）。

3. 將非 Copy 值以值傳遞給新所有者時，原本的綁定不再能使用它，避免兩個擁有者在作用域結束時重複釋放。

```rust run
fn main() {
    let a = String::from("hello");
    let b = a;                     // 移動所有權
    println!("{b}");
    // println!("{a}");            // 若取消註解，會觸發 E0382
}
```

為何 String 賦值不自動複製文字？深層複製會配置新記憶體，成本不小。Rust 選擇預設移動（move），需要複製時再明確請求 `clone()`。對這個非空的 `String`，複製後兩個值分別管理各自的緩衝區。

```rust run
let a = String::from("hello");
let b = a.clone(); // 新配置、複製資料，a 與 b 各自擁有
```

### 從 E0382 找出移動位置

```rust error E0382
fn main() {
    let a = String::from("hello");
    let b = a;
    println!("{a} {b}");
}
```

診斷會指出 `a` 的原始綁定、所有權移到 `b` 的位置，以及最後借用 `a` 的位置。先判斷需求。若只想觀察同一份字串，可用 `let b = &a;`。若兩邊都要獨立修改，再用 `let b = a.clone();`。若修改責任已交給 `b`，後續統一使用 `b`。

## 三、那 i32 為什麼可以重用？

```rust run
let x = 5;
let y = x;
println!("{x} {y}");
```

這可以編譯，因為 i32 實作 Copy：賦值產生獨立的位元複本，沒有需要保護的獨占堆積配置。Copy 不是按名稱區分「基本型別」或「複雜型別」；依型別是否實作該 trait 而定，實作 Drop 的型別不能是 Copy。

## 四、為何要「借用」？

如果每次只想閱讀文字都必須把所有權交出去，程式會很難組合。參考 &T 讓函式暫時存取資料，而不成為該資料的所有者。

```rust run
fn size(s: &String) -> usize { s.len() }
fn main() {
    let name = String::from("Rust");
    let n = size(&name);
    println!("{name} {n}");
}
```

&name 建立不可變參考；參數 s: &String 承諾只透過此借用閱讀。讀完後原所有者 name 仍有效。Rust 常偏好借用 &str 而不是 &String，因為更通用。

## 五、共享閱讀與獨占寫入：為何這樣限制？

```rust run
let mut name = String::from("R");
let r = &name;       // 共享借用
println!("{r}");     // 最後一次使用 r
let w = &mut name;   // 可變借用，與 r 的有效使用期不重疊
w.push('!');
println!("{name}");
```

同一期間可以存在多個共享借用 &T；或者一個獨占的可變借用 &mut T，不能同時有重疊的互斥存取。目的不是限制自由，而是防止「有人正在讀時，別人把資料改動或移走」。

比喻：多個人可同時查看公告欄；要更換內容時，需要在修改期間排除其他讀寫者。比喻的邊界：真實借用不是執行時上鎖，通常在編譯時分析；使用生命週期、別名分析與型別規則確保存取安全。

## 六、借用何時結束？不是機械地等到大括號

Rust 使用非詞法生命週期（NLL）等分析，借用有效期常到它的最後一次使用，而不一定要持續到變數宣告所在區塊結尾。編譯器依實際使用與控制流程判斷。

```rust run
let mut s = String::from("ok");
let view = &s;
println!("{view}");
s.push('!'); // 此時 view 不再被使用
```

最重要的是「有沒有重疊存取」，不是「名字還在不在」。

### 為什麼後移最後一次讀取會觸發 E0502？

```rust error E0502
fn main() {
    let mut s = String::from("Rust");
    let view = &s;
    s.push('!');
    println!("{view}");
}
```

`push` 要借用 `&mut String`，容量不足時可能更換緩衝區。由於 `view` 稍後還要讀取，兩個借用會重疊。把輸出移到 `push` 之前可先結束讀取。若想印更新後的文字，先呼叫 `push`，再建立新的共享借用。

## 七、懸空參考到底危險在哪？

```rust error E0106,E0515
fn broken() -> &String {
    let s = String::from("gone");
    &s
}
```

這段反例先觸發 E0106，因為函式沒有可供推導輸出生命週期的參考參數。就算寫上 `fn broken<'a>() -> &'a String`，仍會觸發 E0515。
原因是 `s` 在函式結束時清理，無法把指向區域資料的參考交給呼叫者。

正確做法 A：回傳擁有的 String。

```rust fragment
fn make_name() -> String {
    String::from("safe")
}
```

正確做法 B：若資料由呼叫者持有，可以回傳借用，並由生命週期約束來源。此例的參考輸入只有一個，Rust 能省略生命週期標註。比較兩個借用輸入的寫法見 [第 10 章](../generics-traits-lifetimes/)。

```rust run
fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("")
}
fn main() {
    let text = String::from("第一行\n第二行");
    assert_eq!(first_line(&text), "第一行");
}
```

## 八、String 與 &str 到底差什麼？

String 是擁有 UTF-8 位元組緩衝區的可成長字串；&str 是指向有效 UTF-8 資料區段的借用切片（指標 + 長度的概念），不一定由自己擁有資料。字面量 "hello" 的型別通常是 &'static str，資料存在程式可使用的靜態儲存區。

```rust fragment
fn greet(name: &str) { println!("你好，{name}"); }
let owned = String::from("Rust");
greet(&owned);
greet("世界");
```

用 &str 當閱讀參數通常能同時接收 String 借用與字串字面量。

## 九、切片解決什麼問題？

假設函式回傳字串中第一個單字的整數索引。若原字串被修改，舊索引可能不再對應正確資料。&str 切片把「哪一段資料」與借用關係綁在一起，讓編譯器追蹤有效性。

```rust run
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

這是按 Unicode 空白分隔的示範；切片以位元組為索引，不能隨意在 UTF-8 字元中間切開，例如 &s[0..1] 對中文字可能在執行時 panic。需要依字元或使用者感知字素處理時應另選 API。

## 十、記憶體圖：把 move 與 borrow 連起來

String 的「指標/長度/容量」是管理資料的值。move 使原綁定失效，把管理責任轉交；clone 建立另一份內容；& 只是受約束地查看；&mut 是受約束地修改。

不要誤以為 move 一定會產生機器層級的位址搬遷：所有權語義與最終機器碼的實際複製、最佳化不是同一件事。也不要認定所有局部值必定位於堆疊：編譯器可最佳化。

## 十一、編譯器診斷讀法

遇到 use of moved value，找出哪個位置消耗了所有權；遇到 cannot borrow as mutable，檢查 mut 與活躍借用；遇到 does not live long enough，追資料主人何時被釋放；不要只用 clone() 掩蓋設計問題。

## 十二、動手驗收與答案

題1：把 String 傳給 fn consume(s: String) 後為何不能再用？因所有權移入函式，呼叫者的綁定無效。

題2：只需閱讀時改成 fn consume(s: &str)，呼叫時 &name；解釋為何較通用。

題3：為何同一段期間不允許一個 &mut 與任何可能存取相同資料的 &？因讀寫衝突可能破壞參考有效性與資料不變條件。

題4：使用 String::clone 後，修改 b 是否會修改 a？不會，兩份擁有獨立緩衝區。

題5：拆解 let first = &s[0..2]，確認範圍是 byte offset，對中文字測試前先閱讀 UTF-8 章節。

終極驗收：能不背口訣，直接從「誰負責釋放、有哪些活躍借用」推導每個借用錯誤。

### 實作挑戰：借用單字之後再清空字串

用上面的 `first_word` 取得「Rust 課程」的第一個單字。在 `println!` 之前呼叫 `clear()`，預測編譯器會拒絕哪一行。再做兩次修改，分別保留原本的切片與取得一份可獨立存活的字串。

**解析。** 第一版把 `println!` 移到 `clear()` 之前，可以在輸出時維持借用，等最後一次使用後再清空。第二版呼叫 `first_word(&title).to_owned()` 建立獨立的 `String`。此時可以先清空 `title` 再印出第一個單字。多出的一次配置與複製換取了獨立的生命週期。

來源：[The Rust Programming Language，第 4 章](https://doc.rust-lang.org/stable/book/ch04-00-understanding-ownership.html)
