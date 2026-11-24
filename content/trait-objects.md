# 第18章｜Rust 為何沒有傳統的類別繼承？

> **戰士與法師怎樣共用行動規則？** <!-- callout:context -->
>
> 想做遊戲中的角色行動系統，戰士、法師和怪物的內部資料各不相同，但都能執行攻擊。若每次新增角色就要重寫所有控制流程，程式很難擴充。Rust 用共同的行為契約和組合來解決這個問題。

## 一、先拆解「物件導向」的三種常見能力

物件導向常被用來指封裝、抽象、依行為多型、繼承等，但它們不是不可分割的一套設計。Rust 使用 struct + impl 組織狀態與方法，透過模組可見性做封裝，以 trait 表達行為，以泛型或 trait object 實現多型；不要求把所有型別塞進一棵繼承樹。

比喻：不同車款不一定需要先被歸入「大車類」再層層繼承；只要符合能「行駛」的共同介面，就能進行同一種操作。代價是需要把不同資料型別與呼叫方式設計清楚。

> **帳戶窗口與共同操作規格** <!-- callout:analogy -->
>
> 封裝像只能透過存款與提款窗口操作的帳戶；Trait 像各種裝置都能符合的操作規格；多型讓同一段呼叫流程可以操作不同型別。這不需要把全部型別放進繼承樹。

## 二、封裝：保護不變條件

```rust fragment
pub struct Account { balance: i64 }
impl Account {
    pub fn new() -> Self { Self { balance: 0 } }
    pub fn deposit(&mut self, amount: i64) -> Result<(), &'static str> {
        if amount < 0 { return Err("不可存負數"); }
        self.balance = self.balance.checked_add(amount).ok_or("金額溢位")?;
        Ok(())
    }
    pub fn balance(&self) -> i64 { self.balance }
}
```

balance 私有，外界只能走經過驗證的入口。實際金融系統還須考量貨幣、交易與持久性，這只是演示封裝如何保護規則。

> **透過 Account 保護餘額** <!-- callout:walkthrough -->
>
> 第一個 Account 範例將 balance 放在私有欄位，`new` 建立初始值，`deposit(&mut self, ...)` 借用並修改帳戶，先拒絕負數，再用 `checked_add` 檢查溢位。`?` 讓錯誤提前回傳，成功才更新餘額。

## 三、靜態分派與動態分派

```rust fragment
trait Draw { fn draw(&self); }
struct Button;
impl Draw for Button { fn draw(&self) { println!("按鈕"); } }
struct Text;
impl Draw for Text { fn draw(&self) { println!("文字"); } }
fn draw_one<T: Draw>(x: &T) { x.draw(); }
```

泛型呼叫常在編譯時針對具體型別單態化，稱靜態分派。若要把不同具體型別放在同一個集合，通常可以使用 trait object：

```rust continuation
// 接續前一段 Draw、Button 與 Text 定義。
fn main() {
    let widgets: Vec<Box<dyn Draw>> = vec![Box::new(Button), Box::new(Text)];
    for w in &widgets { w.draw(); }
}
```

dyn Draw 通過動態分派呼叫方法；通常涉及 vtable 之類的實作機制。不是所有 trait 都能做成 trait object，因方法簽名與 Self 使用方式可能不符合 dyn compatibility 規則。

## 四、何時不該用 dyn？

若只有一兩種型別、效能敏感、需要完整泛型資訊，enum 或泛型常更直接；若集合要容納未知擴充型別，dyn trait 可能更合適。不要以「動態分派會有成本」推論它一定是瓶頸，需衡量設計彈性與實際效能。

> **什麼時候其實不需要 dyn？** <!-- callout:warning -->
>
> `dyn Trait` 是透過執行時選擇實作的動態分派，不代表任何 Trait 都能成為 trait object；仍需符合 dyn compatibility。若已知型別或可能的變體有限，泛型或 enum 往往更簡單。

## 五、組合優於繼承

Rust 可把物件拆成多種獨立能力：Serialize、Display、Draw 等 trait。這降低硬繼承階層的耦合，但也可能帶來大量 trait bounds；好的抽象應從實際重複需求長出來。

## 六、狀態模式：如何表示合法流程？

例如文章有 Draft → Review → Published。可用一個 enum 表示狀態；若希望「未發佈文章根本沒有 publish 後專用的方法」，可使用不同 struct 與消耗 self 的方法，建立型別狀態（typestate）：

```rust program
struct Draft { body: String }
struct Published { body: String }
impl Draft {
    fn new(text: &str) -> Self { Self { body: text.into() } }
    fn publish(self) -> Published { Published { body: self.body } }
}
impl Published {
    fn body(&self) -> &str { &self.body }
}
fn main() {
    let draft = Draft::new("Rust");
    let article = draft.publish();
    println!("{}", article.body());
}
```

編譯器禁止在 Draft 上呼叫 body()，因為該方法根本不存在；且 publish 消耗 Draft，避免原草稿與已發佈物件被同時當成同一資源。

## 七、代價與反例

如果狀態需序列化、從資料庫還原、支援大量動態切換，typestate 可能讓型別數量與介面很複雜。這時使用 enum + 明確驗證未必比較差。模型要配合需求，而不是盲目認為「編譯期狀態一定最好」。

## 八、練習與驗收

題1：把狀態模式擴充成 Draft → Review → Published，禁止 Draft 直接 publish。

題2：用 enum 和 typestate 各實作一次，討論擴充新狀態的成本。

題3：在 Vec 中放不同 Draw 物件，說明為何 Vec<T> 仍然只存一個統一型別 Box<dyn Draw>。

驗收：區分封裝、trait、多型、動態分派、類別繼承與 typestate，不再把「有方法」等同於傳統 OOP。

> **讓不同形狀共用繪圖介面** <!-- callout:practice -->
>
> 設計一個共同的 `draw` 行為，為圓形和方形各自實作。比較使用泛型處理同一具體型別，以及使用 `Vec<Box<dyn ...>>` 保存不同具體型別；說明為何需要兩種設計。

參考：<https://doc.rust-lang.org/stable/book/ch18-00-oop.html>
