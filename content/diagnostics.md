# A02｜學會閱讀 Rust 編譯錯誤

定位：搭配全部章節使用，尤其第4、7、9、10、15、16章。

## 一、診斷流程比背錯誤碼重要

第一步，讀最上面的 error 類型與完整訊息；第二步，看箭頭指向的行列；第三步，追「這個值在什麼地方宣告、移動、借用、修改、釋放」；第四步，畫出最小重現；第五步，修正原本想表達的語意，而非只想讓編譯器閉嘴。

rustc --explain E0382 可以查看錯誤碼說明；cargo check 通常更適合快速反覆驗證。

## 二、E0382：use of moved value

```rust error E0382
let a = String::from("A");
let b = a;
println!("{a}"); // 故意錯誤
```

根因：a 所擁有的資源已轉交給 b，不能同時假裝 a 仍有所有權。

修法選擇：若只需觀察，改為借用；若真的需要兩份獨立擁有的資料，使用 clone；若只需後續使用 b，直接修改程式的資料流。

錯誤修法：看到 E0382 就到處 clone，會掩蓋真正的 API 設計問題。

## 三、E0502：cannot borrow ... as mutable because it is also borrowed as immutable

```rust error E0502
let mut xs = vec![1, 2];
let first = &xs[0];
xs.push(3);
println!("{first}");
```

根因：first 必須保持有效，但 push 可能重配 Vec 緩衝區。修法：先使用 first 再 push；或把所需小值 Copy 到獨立變數；或重新設計借用時間。

## 四、E0499：cannot borrow ... as mutable more than once

```rust error E0499
let mut x = 1;
let a = &mut x;
let b = &mut x;
*a += 1;
*b += 1;
```

根因：兩個活躍且可能衝突的獨占借用。修法：縮短其中一個借用的有效使用區間，或設計串行修改流程。

## 五、E0597：borrowed value does not live long enough

```rust error E0597
let value;
{
    let s = String::from("temporary");
    value = &s;
}
println!("{value}");
```

根因：s 在內層區塊結束就被銷毀，value 不能安全引用它。修法：讓資料真正活得更久、移動所有權到外層，或回傳擁有資料而非參考。

## 六、E0277：trait bound not satisfied

```rust error E0277
fn show<T: std::fmt::Display>(x: T) { println!("{x}"); }
show(vec![1, 2, 3]); // Vec 沒有 Display
```

根因：演算法要求 T 能 Display，但 Vec<i32> 不符合。修法：需求若是除錯輸出，改用 Debug + {:?}；若是自訂人類格式，自己實作顯示邏輯。不能把限制移除卻還使用同樣能力。

## 七、E0308：mismatched types

```rust error E0308
fn count() -> i32 { 3; }
```

根因：分號使最後的 3 變成不回傳值的語句，區塊結果為 ()。其他情況可能是 if 分支型別不一致、String 與 &str 混淆、不同整數型別不相容。

## 八、E0507：cannot move out of borrowed content

當只有 &T 的借用卻試圖取得其中非 Copy 欄位所有權，就會衝突。問自己：我真的需要帶走嗎？可以只借用欄位嗎？擁有者是否允許合法地 replace / take？clone 是否在語意上必要？

## 九、常見執行時問題不等於編譯錯誤

panic：可能由 unwrap None、index 越界等引起。deadlock：鎖順序或互相等待造成。stack overflow：過深遞迴。邏輯結果錯：型別安全不代表符合需求。

執行時錯誤應觀察重現條件、輸入、panic 堆疊資訊與測試；與編譯時錯誤診斷方式不同。

## 十、錯誤到設計改進的對照

Move 錯誤 → 檢查所有權介面。Borrow 錯誤 → 檢查同時存取。Lifetime 錯誤 → 檢查資料來源與存活區間。Trait 錯誤 → 檢查能力契約。Type 錯誤 → 檢查資料意義是否匹配。

## 十一、解題流程練習

把每一種錯誤分別輸入小專案，先不要看範例修法，自己寫出原因與兩種可能設計，再執行 cargo check 驗證。全部錯誤碼請以編譯器當下的實際診斷為準；版本與上下文可能影響代碼與建議。

官方索引：<https://doc.rust-lang.org/error_codes/error-index.html>

Clippy：<https://doc.rust-lang.org/clippy/>
