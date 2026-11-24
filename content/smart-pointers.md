# 第15章｜智慧指標：當一般所有權不能滿足資料結構需求

## 一、為何 String 和 Vec 不是唯一會管理記憶體的型別？

一般參考 &T 是「借用」而不擁有資料；但有時要建立遞迴型別、把一份不可直接放入固定大小結構的資料放到堆積、讓多處共享擁有權，甚至在共享外觀下變更內部狀態。智慧指標通常是擁有特殊管理規則的型別，常會實作 Deref 與 Drop。

比喻：普通 &T 是一張借閱證；Box<T> 是擁有專屬保管箱；Rc<T> 是多個責任人共同持有一份物品的所有權紀錄；RefCell<T> 是需要在執行時檢查借閱衝突的管理櫃台。比喻有界：這些型別的複製與共享成本、執行時檢查規則完全不一樣。

## 二、Box<T> 為什麼和「放在 heap」有關？

```rust run
fn main() {
    let n = Box::new(123);
    println!("{}", *n);
}
```

Box<T> 具有 T 的所有權，通常將 T 放在堆積，Box 值本身保存管理它的指標。離開作用域時會釋放它所擁有的資源。*n 是解參考。

如果只是 i32，使用 Box 通常不是必要優化；它的意義在遞迴型別或其他需要間接層的情況。

## 三、遞迴型別為何必須有固定大小？

```rust fragment
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

若寫 Cons(i32, List)，List 的大小包含另一個 List，而另一個 List 又包含 List，編譯器無法得出有限的型別大小。Box<List> 把遞迴部分變成固定大小的指標，讓整個 enum 可被配置。

## 四、Deref：為何 Box 可像參考一樣使用？

Deref trait 定義如何從智慧指標取得所指向值的參考。Rust 在符合條件時使用 deref coercion，使 &String 可以轉為 &str 等，簡化 API 呼叫。但這不是「任意型別可以強轉」；轉換順序受 trait 實作約束。

```rust run
fn greet(s: &str) { println!("{s}"); }
let owned = Box::new(String::from("hello"));
greet(&owned);
```

## 五、Drop：為何資源可自動清理？

Drop trait 的 drop 方法在值結束生命週期時執行清理工作。清理不是只為了 heap，還可能釋放檔案、鎖或其他資源。通常不能直接呼叫 value.drop() 來強迫銷毀，需用 std::mem::drop(value) 明確消耗它。

## 六、Rc<T>：同一份資源為何能有多位擁有者？

```rust run
use std::rc::Rc;
let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a);
println!("{} {}", a, b);
```

Rc 記錄強參考計數，最後一個強擁有者離開後才釋放內容；Rc::clone 主要增加引用計數，不是 clone 內層 String。Rc 用於單執行緒情境，不能拿去跨執行緒當作 Arc 替身。

## 七、RefCell<T>：為何要執行時借用檢查？

```rust run
use std::cell::RefCell;
let score = RefCell::new(3);
*score.borrow_mut() += 1;
println!("{}", score.borrow());
```

RefCell 允許透過共享外觀取得可變借用，但在執行時檢查「多個不可變借用或一個可變借用」規則；違反時 borrow / borrow_mut 會 panic，try_borrow 系列則回傳 Result。

內部可變性不是繞過所有權安全保證，而是把某些無法在編譯時證明的借用檢查延後到執行時。

## 八、Rc<RefCell<T>> 的取捨

常見於單執行緒圖結構：Rc 共享擁有權，RefCell 管理可變內部狀態。但設計複雜度和執行時檢查成本會增加，應先問能否用明確所有權、索引或資料流取代。

## 九、循環參考為何洩漏？

若 A 以 Rc 強指向 B、B 又強指回 A，即使外部所有者都走了，兩者強計數可能仍不為 0，因此記憶體無法釋放。使用 Weak<T> 表達不負責維持生命週期的非擁有連結，upgrade() 會得到 Option<Rc<T>>，因目標可能已經消失。

常見設計：樹的父指標 Weak、子節點 Rc，避免父子互相保留造成循環。

## 十、Box / Rc / Arc / RefCell / Mutex 選擇表

單一擁有且要間接配置：Box<T>。單執行緒共享擁有權：Rc<T>。跨執行緒共享擁有權：Arc<T>。單執行緒內部可變：RefCell<T>。跨執行緒協調修改：Mutex<T> 常搭配 Arc<T>。

注意 Arc 只讓共享擁有權的參考計數操作具備執行緒安全性，不會自動讓包住的 T 變成安全可任意修改。

## 十一、驗收

題1：說明為什麼遞迴 enum 用 Box 能使大小有限。

題2：Rc::clone 與 String::clone 的主要差異？

題3：同時保持兩個 borrow_mut 為何 panic？如何用 try_borrow_mut 避免直接 panic？

題4：設計樹節點的 Weak 父連結；說明為何升級回 Rc 可能失敗。

驗收：能由「誰擁有、誰可修改、檢查在編譯期還是執行時」選智慧指標，而不是靠感覺套 Rc<RefCell<...>>。

參考：<https://doc.rust-lang.org/stable/book/ch15-00-smart-pointers.html>
