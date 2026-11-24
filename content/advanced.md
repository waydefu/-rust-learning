# 第20章｜安全邊界與語言延伸能力

> **來龍去脈**
>
> 前面多數程式都由編譯器檢查記憶體安全。但作業系統、硬體或其他語言的介面，有時需要操作編譯器無法完整證明安全的資料。Rust 提供受限制的 unsafe 邊界，讓這些操作能被清楚標示及審查。

## 一、先澄清：unsafe 不等於關閉所有檢查

Rust 的 unsafe 允許開發者執行某些編譯器無法自行驗證安全性的操作，並承擔額外證明責任。一般所有權與型別檢查並不因此全部關閉。

把 unsafe 想像成工程師可進入高壓電機房維修，並不等於整棟建築不再需要安全規範。真正的責任是建立前置條件與不變條件，使安全 API 的呼叫者不用面對未定義行為。

> **生活比喻**
>
> unsafe 像維修人員進入機房時必須自行確認斷電與工具安全：允許特殊操作不等於免除其他規則。原始指標（raw pointer）保存地址等資訊，但不像參考一樣提供同等的有效性保證。

## 二、unsafe 的主要能力與邊界

常見 unsafe 操作包括：解參考原始指標、呼叫 unsafe 函式或方法、存取可變 static、實作 unsafe trait、存取 union 的欄位。語言版本也會影響 unsafe extern 等語法細節，應對照現行 Rust Reference。

```rust program
fn main() {
    let mut n = 5;
    let ptr: *mut i32 = &mut n;
    unsafe {
        *ptr += 1;
    }
    println!("{n}");
}
```

這段由有效 &mut i32 建立原始指標並在有效期間使用，是可解釋的基本示範；但 raw pointer 不具有普通參考同等的編譯期有效性保證，未來若改動存活期間或別名關係，就可能變成 UB（未定義行為）。

> **程式拆解**
>
> `let ptr: *mut i32 = &mut n;` 將可變參考轉為原始指標。`unsafe { *ptr += 1; }` 解參考並修改 n，必須確認指標在使用時有效、對齊且存取符合別名規則。這個範例結束後印出 6；普通安全 Rust 就能完成加一，不必為了簡單操作使用 unsafe。

## 三、為何 FFI 需要 unsafe？

呼叫 C 等外部程式庫時，Rust 編譯器無法保證對方會遵守 Rust 的別名、資料佈局與記憶體有效性條件，因此邊界常要求 unsafe 宣告與呼叫。

#[repr(C)] 可讓部分結構型別遵循 C 相容佈局規則，但並不是「任何 Rust 型別都能直接交給 C」。還需檢查欄位型別、字串 NUL 結尾、對齊、函式 ABI、錯誤碼、釋放責任、跨語言例外與執行緒規則。

## 四、如何審核一段 unsafe？

先寫安全條件：指標是否非空、對齊、指向足夠可用位元組、初始化、生命週期有效、無衝突別名、無資料競爭；然後追蹤每條路徑是否維持條件。

最好把 unsafe 侷限在最小區塊，外面提供有明確前置條件或完整自行驗證的安全 API。即使編譯成功，unsafe 程式仍可能有 UB，因此需要額外測試與程式碼審查。

> **常見陷阱**
>
> unsafe 不會關掉借用檢查或型別系統，也不會修好錯誤的所有權設計。只有當安全 Rust 的介面無法表達必要操作時才考慮它，並將安全前置條件寫清楚，盡可能縮小範圍。

## 五、進階 Trait：關聯型別、預設方法與完全限定語法

```rust fragment
trait IteratorLike {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

關聯型別（associated type）把實作與「該實作輸出的元素型別」連起來，與泛型參數的角色不同。若多個 trait 有同名方法，可使用 <Type as Trait>::method(...) 做明確選擇。

進一步的泛型關聯型別（GAT）、高階 Trait Bound（HRTB）與 trait object 相容性，是設計高抽象函式庫時需要了解的工具；初學時先把基本 trait 約束和生命週期掌握。

## 六、型別別名、never、newtype

type Id = u64; 只是別名，不是新的型別邊界；struct UserId(u64); 則建立獨立型別，防止把 UserId 與 OrderId 不小心混用。

!（never type）描述永遠不正常返回的計算，例如 panic! 或無限循環；它與回傳空值的 () 不相同。某些進階型別用法可能有穩定性限制，請對照當前編譯器與 Reference。

## 七、巨集與函式到底有什麼不同？

函式收到的是已形成的值；巨集在編譯過程中操作 token 或語法結構，可生成程式碼。這使 println!、vec! 等具備不同於普通函式的表達能力。

```rust program
macro_rules! say_hi {
    ($name:expr) => {
        println!("你好，{}", $name);
    };
}
fn main() { say_hi!("Rust"); }
```

$name:expr 表示匹配一個運算式語法片段；巨集展開可能影響名稱解析、語意及診斷。宣告式 macro_rules! 與程序式巨集（derive、attribute、function-like）是不同機制，不要把兩者混成一種魔法。

> **動手驗收**
>
> 先把原始指標範例改成普通 `&mut`，得到相同輸出。再閱讀 FFI 範例，列出呼叫前必須確認的指標、長度、有效期等條件；只要無法證明條件，就不要執行危險操作。

## 八、實作與驗收

練習1：建立 UserId 與 OrderId 兩個 newtype，確認不能互相直接指派。

練習2：不用 unsafe 重寫能安全做到的功能；只在確實需要原始指標或 FFI 時引入 unsafe。

練習3：解釋為何 #[repr(C)] 不能讓所有資料直接安全交給 C。

練習4：為一段 unsafe API 寫 # Safety 前置條件文件；明確指出呼叫者必須保證哪些事。

驗收：能區分安全 Rust 能保證的部分與 unsafe 開發者必須自行證明的條件；理解 trait、newtype、巨集各解決哪類抽象問題。

參考：<https://doc.rust-lang.org/stable/book/ch20-00-advanced-features.html>
