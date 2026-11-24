# 第07章｜程式長大後，如何避免全部擠在 main.rs？

> **來龍去脈**
>
> 當作業只有一張紙，什麼都寫在一起沒有問題；當你有數百頁筆記，就需要按科目分區、建立目錄和規則。程式變大也是如此：如果名字重複、資料到處都能改，合作和除錯會越來越難。

## 一、問題來源：名稱衝突、責任混雜、外部依賴

十行程式可以放一個檔案；一萬行程式如果全部同層，任何人都能改任何資料，便很難定位、測試或維護。Rust 用 package、crate、module、路徑、可見性解決不同層次的問題。

比喻：package 是整個建案；crate 是可單獨編譯的建築單位；module 是樓層與房間；pub 是允許外部進入的門，不是「所有地方都公開」的快捷鍵。

> **生活比喻**
>
> package 是 Cargo 管理的專案單位；crate 是 Rust 的編譯單位，可以是執行檔或函式庫；module 是 crate 內組織名稱與可見性的區域。三者不是同義詞，一個 package 可以包含多個 crate。

## 二、四個名詞別混在一起

Package：Cargo.toml 描述的 Cargo 專案單位，可包含相應的 crate 目標。Crate：編譯單元，通常有一個 crate root；src/main.rs 是一般 binary crate 的根，src/lib.rs 是一般 library crate 的根。

Module：crate 內的命名空間、私有性邊界，不等於一定一個檔案。Path：例如 crate::game::Player，用來精確定位項目。

Crates.io：公開發佈與下載套件的註冊中心；Cargo 是管理依賴和建構的工具。這四者不是同義詞。

> **程式拆解**
>
> `pub mod game;` 宣告一個名為 game 的公開模組，對應的檔案安排要符合 Rust 規則。`::` 表示路徑中的一層；`use` 把路徑中的名稱帶入目前作用域，讓呼叫比較簡短，不是把檔案文字複製過來。

## 三、最小可維護專案

假設有 src/lib.rs：

```rust project my_game:src/lib.rs
pub mod game;
```

在 src/game.rs：

```rust project my_game:src/game.rs
pub struct Player { pub name: String, hp: u32 }
impl Player {
    pub fn new(name: String) -> Self { Self { name, hp: 100 } }
    pub fn hp(&self) -> u32 { self.hp }
}
```

在 src/main.rs：

```rust project my_game:src/main.rs
use my_game::game::Player;
fn main() {
    let p = Player::new("Ada".to_string());
    println!("{} {}", p.name, p.hp());
}
```

注意 my_game 必須與 Cargo.toml 中的 crate 名稱相符（package 中的連字號慣例轉為底線）。hp 是私有欄位，外部不可寫 p.hp；這讓模組有機會維護 hp 的不變條件。

## 四、為何預設私有？

如果所有欄位與函式都公開，未來改內部實作會影響太多呼叫者。私有是隔離變更成本的工程設計。pub 控制路徑中的可見性；有權存取某個項目，仍需要整條可見路徑都可達。

pub(crate) 允許同 crate 使用；pub(super) 可限制在父模組範圍。crate:: 從根開始、self:: 指目前、super:: 指父模組。

> **常見陷阱**
>
> 公開 `pub` 不等於安全，也不代表應將所有欄位公開。內部資料預設私有，可透過受控方法維持合理狀態；把欄位全公開可能使外部任意改壞資料。

## 五、use 不是搬運程式碼

use std::collections::HashMap; 只是讓名字可簡寫。它不會把原始碼貼到目前檔案，也不是 npm 或 Python 的動態 import；編譯器將路徑解析成正確的項目。

as 能重新命名避免衝突，例如 use std::io::Result as IoResult;。重導出 pub use 可建立穩定的公開 API 表面，讓內部資料夾調整不必迫使呼叫者改路徑。

## 六、與檔案系統的關係

mod game; 告訴編譯器該模組內容在對應的 game.rs 或 game/mod.rs 等位置；內嵌 mod game { ... } 則直接在同檔定義。模組樹與實體檔案樹相關，但不是完全同一概念。

不要從「同一資料夾就能直接互相呼叫」推理 Rust。正確做法是先畫 module tree，再看每個節點的可見性與路徑。

## 七、Cargo 的 package 與 workspace

一個 package 可能同時有 lib 與多個 binary；多個 package 可以用 Cargo workspace 共用 lockfile 與目標資料夾等資源，第14章詳解。使用外部 crate 前應檢查版本、授權、安全性、維護情況，而不是盲目 cargo add。

> **動手驗收**
>
> 把角色與戰鬥規則放進不同模組，讓 main 只呼叫公開入口。先故意呼叫一個私有項目，閱讀編譯錯誤，再決定公開方法還是保持私有。

## 八、實作／驗收

練習：將上一章的 QuestState 移到 game.rs，從 main.rs 使用；只公開「建構任務與查詢狀態」的必要 API。

錯誤實驗：取消 pub mod game 或取消 pub struct Player，讀編譯器的 private module / private struct 診斷。

驗收：用自己的話解釋 crate、package、module、檔案、use、pub、pub(crate) 及為何隱藏內部細節有利長期維護。

參考：<https://doc.rust-lang.org/stable/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html>
