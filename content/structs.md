# 第05章｜從散亂資料到一致的結構

## 一、為什麼陣列與 tuple 不夠？

遊戲角色有名稱、血量、等級。若用三個互不相干的變數，函式容易收錯順序；tuple 雖能綁一起，但 user.0 不易維護。struct 讓資料具備可命名欄位，讓程式描述「角色」這個概念。

```rust program
struct Player {
    name: String,
    hp: u32,
    level: u32,
}
fn main() {
    let mut p = Player { name: String::from("Ada"), hp: 100, level: 1 };
    p.hp = 80;
    println!("{}: {}", p.name, p.hp);
}
```

## 二、為何要有欄位與可變性

struct 是型別藍圖，Player { ... } 是值。整個變數必須宣告 mut 才能改一般欄位；Rust 預設不支援只讓某個一般欄位 mut，這維持對值變更權限的一致理解。

欄位初始化簡寫：若區域變數也叫 name，可以寫 Player { name, hp: 100, level: 1 }。結構更新語法 Player { hp: 90, ..old } 可能把舊值的非 Copy 欄位移動到新值，使 old 部分不能再用；這是第4章規則的直接延伸。

## 三、Tuple struct、單元 struct

struct Color(u8, u8, u8); 看似 tuple，但 Color 是不同於其他同形 tuple 的專屬型別，適合表示不該混用的概念。struct Marker; 是零欄位的單元型別，可用於型別層級標記。

## 四、讓資料有行為：impl

```rust program
struct Rectangle { width: u32, height: u32 }
impl Rectangle {
    fn area(&self) -> u32 { self.width * self.height }
    fn grow(&mut self, by: u32) { self.width += by; }
    fn square(size: u32) -> Self {
        Self { width: size, height: size }
    }
}
fn main() {
    let mut rect = Rectangle::square(4);
    rect.grow(2);
    println!("{}", rect.area());
}
```

impl 把與型別相關的函式組織在一起；fn area(&self) 是方法，借用當前實例；&mut self 需要修改實例；沒有 self 的 square 是關聯函式，透過 :: 呼叫。

比喻：struct 是角色資料卡，impl 是這類角色會執行的操作手冊。兩者不是物件必須共享的魔法欄位，Rust 方法仍可使用普通函式概念理解。

## 五、self 的三種接收方式

&self 只讀借用；&mut self 獨占可變借用；self 按值接收並可能消耗原物件。選擇取決於是否需要所有權，而不是單純看函式名稱。

## 六、Debug、派生與封裝

```rust fragment
#[derive(Debug)]
struct Point { x: i32, y: i32 }
let p = Point { x: 1, y: 2 };
println!("{p:?}");
```

#[derive(Debug)] 請編譯器產生實作 Debug 的程式碼。為何不預設所有型別都能印？因格式與資料可見性涉及型別的明確契約。

使用 mod 與 pub 控制封裝會在第7章學到。struct 不會自動保證 hp 不超過上限；要建立不變條件，通常透過私有欄位與建構函式檢查。

## 七、常見錯誤／答案

錯誤：println!("{p}") 無法印出自訂 struct；它沒有 Display。改 Debug + {:?} 或自己實作 Display。

錯誤：未宣告 mut 就 p.hp = 5；Rust 不允許修改不可變綁定的欄位。

練習：設計 Monster { name, hp, attack }，新增 fn hit(&mut self, damage:u32)，使用 saturating_sub 避免無號減法溢位。

驗收：能區分型別、值、欄位、方法、關聯函式、接收 self 的三種方式，以及 struct update 導致的部分移動。

參考：<https://doc.rust-lang.org/stable/book/ch05-00-structs.html>
