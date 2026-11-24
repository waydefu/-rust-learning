# 第10章｜從重複程式，到安全且可組合的抽象

> **同一個找最大值的規則，能不能重用？** <!-- callout:context -->
>
> 你要找三個分數中的最大值，也要找三個字母中的最大值。如果每種資料都寫一遍相同流程，以後改規則就要改好幾份。泛型讓流程可以重用，但還要說清楚資料必須具備什麼能力，參考又能有效到哪裡。

## 一、泛型解決什麼問題？

如果找最大值的演算法要對 i32、u32、char 各寫一份，維護重複又容易分歧。但若硬把任何型別都視為能比較，也是不正確的。泛型是「同一個演算法，對符合某些條件的型別成立」。

```rust program
fn largest<T: Ord>(list: &[T]) -> Option<&T> {
    let (first, rest) = list.split_first()?;
    let mut max = first;
    for item in rest {
        if item > max { max = item; }
    }
    Some(max)
}
fn main() {
    println!("{:?}", largest(&[3, 7, 1]));
}
```

<T: Ord> 限制 T 必須支援全序比較；回傳 Option<&T> 避免空列表與不必要複製。這份程式也示範了從參數引用推導回傳引用的省略生命週期。

> **largest 的參數與回傳值** <!-- callout:walkthrough -->
>
> `fn largest<T: Ord>` 宣告 T 必須可全序比較；`list: &[T]` 借用一段 T；`Option<&T>` 可能找不到最大值（空切片），有值時回傳借用而非複製。`split_first()?` 先取得第一項與剩餘部分，空切片時直接回傳 None。

## 二、Trait 是行為契約，不是父類別

Trait 說明「實作某種能力需要哪些方法」。例如 Display 表示某型別能以人類可閱讀格式顯示；Clone 表示可明確複製。

```rust fragment
trait Describable {
    fn describe(&self) -> String;
}
struct Weapon { name: String, attack: u32 }
impl Describable for Weapon {
    fn describe(&self) -> String {
        format!("{} 攻擊 {}", self.name, self.attack)
    }
}
fn show<T: Describable>(x: &T) { println!("{}", x.describe()); }
```

這表示任何型別只要遵守 Describable 契約就可使用 show，而不要求它們從某個共同物件階層繼承。Rust 的多型可透過泛型（通常靜態分派）或 trait object（動態分派）達成，兩者有不同成本與表達限制。

> **型別的空格、能力與借閱期限** <!-- callout:analogy -->
>
> 泛型 `T` 像尚未決定內容的空白欄位，Trait 像能力證明：需要能比較就要求 Ord。生命週期像借閱規定，表示參考必須和來源保持有效關係；標註不會延長資料的存活時間。

## 三、Trait bound 為什麼不能省略？

編譯器必須知道泛型函式內哪些運算合法。若寫 fn f<T>(a:T,b:T) { a > b }，T 不一定有 >；沒有足夠約束時編譯器無法保證。

where 語法只是讓多個限制更可讀：

```rust fragment
fn equal_text<T, U>(a: &T, b: &U) -> bool
where T: std::fmt::Display, U: std::fmt::Display {
    a.to_string() == b.to_string()
}
```

這個例子比較顯示文字，不一定等同於型別本身的語意相等；正式設計應選合適的 PartialEq 等契約。

## 四、生命週期到底在解決什麼？

所有權讓資源在適當時機清理；借用提供不取得所有權的存取方式；於是就有新問題：如果函式回傳某個參考，這個參考究竟指向哪個輸入資料？能保證它仍然有效嗎？

```rust fragment
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}
```

'a 不是幫資料延長壽命的魔法，也不是執行時倒數計時器。它表達「輸入參考與輸出參考應滿足的借用關係」。實際可用期限由輸入中較受限制的有效範圍決定。

比喻：兩張入場券各有自己的有效期限，函式可交回其中一張；如果不知道交回哪張，就必須以同時適用的較保守權限承諾，而不是自己蓋一張永久通行證。

## 五、為什麼編譯器不能永遠自動推導？

當函式只有單一借用輸入時，Rust 的省略規則通常可把回傳參考聯繫到它；但兩個輸入都可能成為回傳值時，就需要讓 API 明確告訴呼叫者關係。

```rust fragment
fn identity(s: &str) -> &str { s }
// 一個輸入，輸出生命週期可省略
```

這並不表示生命週期消失；只是編譯器依規則補上。也不要把 'static 誤用成「修正借用錯誤」：&'static str 要求參考所指資料在程式所需的整段靜態生命期有效，普通區域 String 無法隨意變成它。

> **標上生命週期不會延長資料的壽命** <!-- callout:warning -->
>
> `'a` 不是指定活幾秒，也不是讓區域變數變成永久資料。兩個輸入與輸出共享同一生命週期參數時，呼叫者只能在兩者都有效的範圍使用結果；不能用標註修補懸空參考。

## 六、反例：不可能的參考

```rust error E0515
fn bad<'a>() -> &'a str {
    let s = String::from("hello");
    &s
}
```

這段必然無法編譯：s 出了函式就被釋放，回傳參考不能借由在簽名上貼 'a 而變安全。正確的設計應回傳 String，或讓資料由呼叫者持有並借入函式。

## 七、結構體持有參考

```rust program
struct Excerpt<'a> { text: &'a str }
fn main() {
    let book = String::from("第一章：開始");
    let e = Excerpt { text: &book };
    println!("{}", e.text);
}
```

Excerpt 借用外部資料，不能在 book 銷毀後繼續使用 e.text。若設計需求是讓 Excerpt 自己長期獨立存在，就考慮改為擁有 String，而不是到處加生命週期參數。

## 八、深入分派：靜態與動態的取捨

fn draw<T: Drawable>(x: &T) 多數情況經單態化為具體型別版本；fn draw(x: &dyn Drawable) 透過 trait object 進行動態分派。dyn 值通常透過指標參考使用，需符合 trait object 可使用性的規則。不要僅憑「動態比較慢」選型，應以彈性、二進位體積、效能測量與 API 設計一起權衡。

## 九、練習／驗收

題1：把對 i32 寫的 max 函式泛化到所有 Ord 類型，並用空 Vec 測試 None。

題2：替 Quest 和 Weapon 實作同一個 Summary trait，寫 show<T: Summary>。

題3：寫 fn first<'a>(x: &'a str, _y: &str) -> &'a str { x }，說明為何第二參考不需綁到輸出。

題4：比較回傳 String、&str、Cow<str> 的所有權與呼叫者負擔；Cow 留到進階。

驗收：看到 lifetime error 時，先畫所有權來源、借用區間、輸出參考關係，不依靠亂加 'static 或 clone 來蒙混。

> **找整數、字元與空清單的最大值** <!-- callout:practice -->
>
> 先以整數、字元、空切片呼叫 largest，預測各自的結果。再寫一個只接受單一 `&str` 並回傳其中一部分的函式，說明回傳參考為何不能比輸入資料活得更久。

參考：<https://doc.rust-lang.org/stable/book/ch10-00-generics.html>
