# 第08章｜集合型別：資料量不固定時怎麼辦？

> **來龍去脈**
>
> 背包開始只有兩件物品，但打怪後會撿到更多。固定長度的陣列不方便應付未知數量，於是需要能成長的集合。不同的集合像不同的收納工具：按順序找物品、按名稱找數量，選擇不一樣。

## 一、固定大小和動態大小的衝突

陣列 [T; N] 長度固定、同元素型別；但輸入紀錄、背包清單、玩家名單的數量事前不知道。Vec<T> 把資料儲存在可動態擴張的緩衝區，是處理可變長度序列的常見選擇。

```rust program
fn main() {
    let mut bag = vec!["藥水", "短劍"];
    bag.push("護盾");
    for item in &bag { println!("{item}"); }
    let first = bag.get(0);
    println!("{first:?}");
}
```

Vec 可以透過 push 增加資料；當容量不足時可能重新配置緩衝區，因此持有指向舊元素的參考同時修改 Vec 會有風險。Rust 的借用規則因此限制以下寫法：

```rust fragment
let mut v = vec![1, 2, 3];
let first = &v[0];
// v.push(4);         // 如果後續還要用 first，這裡將衝突
println!("{first}");
```

比喻：電影院座位滿時要搬到更大的影廳；舊座位編號可能已不指向原來位置。這只是說明「重新配置會使原指標失效」，不是 Vec 每次 push 都一定搬家。

> **生活比喻**
>
> `Vec<T>` 像按順序排列的抽屜，T 表示每格所放的型別；`String` 像可以增加內容的文字緩衝區；`HashMap<K, V>` 像以物品名稱（key）查數量（value）的索引簿。HashMap 不保證按照插入順序排列。

## 二、索引與 get 的邏輯區別

v[100] 越界會 panic；v.get(100) 回傳 Option<&T>，讓程式明確處理不存在。選擇哪個不是風格：若索引不合法是不可接受的程式錯誤，可用 []；若使用者輸入可能超範圍，應用 get。

Vec 存同種元素。若要混放形態不同但屬於同一業務概念的資料，用 enum 當元素型別，而非任意型別亂塞。

> **程式拆解**
>
> `vec!["藥水", "短劍"]` 建立兩項的向量，`push` 在末尾新增護盾，`for item in &bag` 逐一借來讀而不拿走背包。`bag.get(0)` 查詢第一項，回傳 Option；索引從 0 開始。

## 三、String 為何比想像中難？

Rust 的 String 與 &str 都表示 UTF-8 文字，但 String 擁有可變長度緩衝區，&str 是文字切片。UTF-8 每個 Unicode scalar value 可能占 1 到 4 位元組，不是所有文字都能用一個整數索引當第 n 個字。

```rust fragment
let s = String::from("A中😀");
println!("{}", s.len());                 // 位元組數：8
println!("{}", s.chars().count());       // Unicode scalar value 數：3
for b in s.bytes() { println!("{b}"); }  // 位元組
for c in s.chars() { println!("{c}"); }  // Unicode scalar values
```

若要人類看到的「字素叢集」，單純 chars() 仍可能不夠，例如帶組合符號的字或多碼位 emoji。需要時使用適合的 Unicode segmentation 函式庫，明確比較 byte / char / grapheme 三層。

## 四、為何不允許 s[0]？

因為索引 0 究竟是第 0 個位元組還是第 0 個字？若按位元組索引，可能切進某個中文字內部，產生無效 UTF-8。Rust 不提供讓人誤解的字串單整數索引，而要求明確使用 bytes、chars 或切片。

```rust fragment
let hello = "你好";
let first = &hello[0..3];
println!("{first}");
```

這裡 0..3 是 UTF-8 位元組邊界。若寫 0..1，程式會在執行時 panic，不是取得「你」的部分。

> **常見陷阱**
>
> Rust 字串以 UTF-8 位元組儲存，中文字可能占三個位元組，不能用 `s[0]` 猜出第一個字。`get` 可用來安全查找向量索引；若持有集合內的參考又嘗試修改集合，還要遵守借用規則。

## 五、HashMap 解決的是另一種查詢

Vec 適合依順序與索引；若查詢是「名稱 → 分數」，每次從頭搜尋列表的成本容易隨資料量增加。HashMap<K,V> 用雜湊將鍵映射到儲存位置，通常可提供平均接近常數時間的查詢，但不保證所有情況，也不保證迭代順序。

```rust program
use std::collections::HashMap;
fn main() {
    let mut scores = HashMap::new();
    scores.insert("A", 10);
    scores.insert("B", 8);
    *scores.entry("A").or_insert(0) += 1;
    println!("{:?}", scores.get("A"));
}
```

entry() 取得鍵對應的佔位入口；or_insert(0) 在不存在時放入預設值，回傳對值的可變參考。前面的 * 是解參考，才能修改實際分數。

## 六、所有權會追進集合裡

```rust fragment
use std::collections::HashMap;
let key = String::from("玩家");
let mut map = HashMap::new();
map.insert(key, 10);
// println!("{key}"); // key 所有權已被搬進 map
```

不是 HashMap 偷走變數，而是 insert 的參數類型要求取得 K 的所有權；是否移動要看傳入值是否 Copy。

## 七、選擇資料結構的判準

按順序新增、迭代、依位置索引：Vec；UTF-8 文字：String / &str；依鍵查值：HashMap。選型要同時想資料量、存取方式、順序要求、所有權及效能。需要排序鍵時可考慮 BTreeMap。

> **動手驗收**
>
> 建立三項背包，試查第 0、2、3 項並預測 Some 或 None。再用 HashMap 記錄每種物品的數量，思考為什麼重複撿到藥水時更新數量比新增一個同名鍵合理。

## 八、練習與驗收

寫一個文字統計器：輸入 "red blue red" 後，用 split_whitespace 取單字，用 HashMap<String, usize> 累計出現次數。提示：for word in text.split_whitespace() { *count.entry(word.to_string()).or_insert(0) += 1; }。

故意執行 v[99] 再改 v.get(99)。比較「程式中止」與「讓呼叫者選擇如何處理」。

驗收：能解釋 Vec 為何可能重配置、String.len 為何回傳位元組長度、HashMap 為何不能依插入順序索引，以及集合操作何時移動所有權。

參考：<https://doc.rust-lang.org/stable/book/ch08-00-common-collections.html>
