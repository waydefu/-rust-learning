# 第13章｜少寫迴圈，不代表少理解執行

> **選出傷害、轉換數值，再加總** <!-- callout:context -->
>
> 假設你有一整排戰鬥紀錄，只想留下傷害大於 0 的項目、把傷害加倍、最後求總和。每次都重寫迴圈會有許多重複步驟。迭代器提供一致的逐項取值方式，閉包讓你把自己的規則交給它。

## 一、問題：為何常見處理流程容易重複？

「先挑選、再轉換、最後合計」每次寫 for 迴圈都需要重複控制變數。迭代器把「如何取得下一個元素」抽象成共通介面，閉包則是能捕獲周圍環境的匿名函式。

比喻：迭代器像輸送帶，每次有人要求才送下一個物件；閉包像可帶著局部作業規則的檢查員。比喻的限制：真實迭代器可能借用、消耗或產生資料，且不一定是背後已有的集合。

> **輸送帶每次送來一個元素** <!-- callout:analogy -->
>
> 迭代器（iterator）像一條一次交出一件物品的輸送帶；`next()` 取得下一項，沒有就得到 None。閉包（closure）是沒有必須取名字的可呼叫值，還可以依使用方式捕獲外部變數。

## 二、迭代器的核心是 next()

```rust run
let nums = vec![1, 2, 3];
let mut it = nums.iter();
assert_eq!(it.next(), Some(&1));
assert_eq!(it.next(), Some(&2));
```

Iterator trait 定義 Item 關聯型別及 next() -> Option<Self::Item>。Some 表示有下一項，None 表示已結束。每次呼叫 next 才讓進度往前。

iter() 借用元素；iter_mut() 可變借用元素；into_iter() 取得對集合的擁有權並依其實作消耗或移動元素。這三者不是單純寫法不同，所有權效果直接不同。

> **next 如何一步一步前進** <!-- callout:walkthrough -->
>
> `nums.iter()` 建立借用向量的迭代器，`let mut it` 允許更新它的走訪進度，第一次 `it.next()` 得到 Some(&1)，第二次 Some(&2)。`&1` 表示參考；元素仍由原向量管理。

## 三、惰性計算與消費者

```rust run
let values = vec![1, 2, 3, 4];
let processed: Vec<i32> = values.iter()
    .filter(|&&x| x % 2 == 0)
    .map(|&x| x * 10)
    .collect();
assert_eq!(processed, vec![20, 40]);
```

filter、map 是轉換器（adapter），通常先建立迭代器管線，不會立刻跑完。collect、sum、fold、for 會消費迭代器並推動工作。別因為寫了 map 就以為副作用已發生。

> **寫了 map，計算卻還沒有開始** <!-- callout:warning -->
>
> `map`、`filter` 常只是建立惰性的轉接器（adapter），不會立刻把每項做完；`collect`、`sum` 等消費者才驅動走訪。`iter()`、`iter_mut()`、`into_iter()` 分別對應共享借用、可變借用、依值取得元素等不同所有權需求。

## 四、閉包跟函式有何不同？

```rust run
let offset = 3;
let add_offset = |x: i32| x + offset;
assert_eq!(add_offset(7), 10);
```

閉包可捕獲所在作用域的 offset；普通 fn 函式不會自動捕獲周遭局部變數。編譯器依使用情況決定捕獲方式：不可變借用、可變借用或移動。

Fn、FnMut、FnOnce 表達閉包可被呼叫的方式；關鍵在捕獲與使用資源的方式，而不是閉包「是否有回傳值」。

## 五、三種閉包能力的根本

FnOnce 至少能呼叫一次，可能在呼叫時消耗已捕獲的值。FnMut 允許改變捕獲的環境，需要以可變方式呼叫。Fn 可透過共享借用反覆呼叫而不需要獨占修改。

```rust run
let mut count = 0;
let mut inc = || { count += 1; };
inc();
inc();
println!("{count}");
```

move 閉包要求捕獲環境以值方式帶入閉包，常用在執行緒等需要獨立持有資料的場合；不代表閉包內每次呼叫都必定移動所有捕獲值。

## 六、零成本抽象是否等於無成本？

Rust 的泛型與迭代器常透過單態化、內聯與最佳化產生接近手寫迴圈的機器碼；但不是「任何迭代器都一定比手寫快」。效能需考慮演算法、記憶體配置、編譯設定及實際 benchmark。

閉包與迭代器的最大價值還包括意圖清楚、組合方便、降低重複錯誤。

## 七、實作：戰鬥數據統計

```rust run
let damage = [12, 30, 8, 41];
let high: Vec<_> = damage.iter().copied().filter(|x| *x >= 20).collect();
let total: i32 = high.iter().sum();
println!("{high:?}，總傷害 {total}");
```

copied() 將 &i32 轉成 i32，因 i32 是 Copy；對 String 不能同樣無成本複製內容。

## 八、練習與答案

題1：filter 再 map 和 map 再 filter 哪些情況不相同？當過濾條件依原始值或轉換後值不同時結果改變。

題2：從 Vec<String> 生成每個文字的長度 Vec<usize>，使用 iter().map(|s| s.len()).collect()。

題3：自行寫 for 迴圈實現上面資料統計並比較可讀性。

題4：閉包捕獲 String 後移進 thread，為何主執行緒不能繼續任意使用原值？回到所有權。

驗收：能預測什麼時候開始實際迭代、區分迭代器三種取用方式、理解 Fn/FnMut/FnOnce 與 move capture。

> **用兩種寫法得到同一個結果** <!-- callout:practice -->
>
> 對 [1, 2, 3, 4] 先挑偶數，再乘 10，再加總，預測結果 60。用 for 和迭代器各寫一次，說明兩者在哪個步驟開始真正處理元素。

參考：<https://doc.rust-lang.org/stable/book/ch13-00-functional-features.html>
