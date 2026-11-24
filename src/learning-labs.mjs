import { escape } from './course.mjs';
import { icon } from './icons.mjs';

export const ownershipSteps = [
  { code: 'let a = String::from("Rust");', source: 'a: String', target: 'b 尚未宣告', owner: 'a', message: 'a 擁有 String，負責在正常離開作用域時釋放其緩衝區。' },
  { code: 'let b = a;', source: 'a 已移動', target: 'b: String', owner: 'b', message: 'b 取得 String 所有權。堆積的四個位元組沒有因這次賦值被複製。' },
  { code: 'let view: &str = &b;', source: 'view: &str', target: 'b: String', owner: 'b', borrowed: true, message: 'b 維持所有權。view 含資料指標與長度，可以讀取同一段 UTF-8 位元組。' },
  { code: 'drop(b); // view 已無後續使用', source: 'view 已用完', target: 'b 已移動', owner: null, message: 'drop 消耗 b，釋放 String 的配置。如果 view 仍有後續使用，編譯器會在這一步拒絕移動 b。' },
];

export function ownershipLab() {
  return `<section class="lab" data-ownership>
  <p class="lab-kicker">${icon('layers', 17)} 互動記憶體圖</p>
  <h2 id="ownership-lab" tabindex="-1">跟著 String 的所有權走</h2>
  <p>每一步都先猜誰負責釋放緩衝區，再切換狀態。圖上只呈現語意上的綁定與配置。</p>
  <div class="lab-controls" data-steps hidden>${['建立', '移動', '借用', '釋放'].map((label, index) => `<button type="button" data-step="${index}" aria-controls="memory-state" aria-pressed="${index === 0}">${index + 1} ${label}</button>`).join('')}</div>
  <div class="lab-visual" id="memory-state" data-owner="a">
    <code data-lab-code>${escape(ownershipSteps[0].code)}</code>
    <div class="memory-grid" aria-hidden="true">
      <div><span class="memory-label">綁定</span><div class="memory-node" data-binding-a>a: String</div><div class="memory-node" data-binding-b>b 尚未宣告</div></div>
      <svg viewBox="0 0 86 114" preserveAspectRatio="none" focusable="false"><defs><marker id="memory-arrow" markerWidth="7" markerHeight="8" refX="5" refY="4" orient="auto"><path d="m0 1 5 3-5 3" fill="none" stroke="context-stroke"/></marker></defs><path data-line-a d="M1 43C38 43 38 66 82 66" marker-end="url(#memory-arrow)" /><path data-line-b d="M1 90C38 90 38 66 82 66" marker-end="url(#memory-arrow)" /></svg>
      <div><span class="memory-label">堆積緩衝區</span><div class="memory-bytes"><span>R</span><span>u</span><span>s</span><span>t</span></div></div>
    </div>
    <p class="lab-message" data-lab-message aria-live="polite">${ownershipSteps[0].message}</p>
  </div>
  <noscript><ol>${ownershipSteps.map(step => `<li><code>${escape(step.code)}</code> ${step.message}</li>`).join('')}</ol></noscript>
</section>`;
}

const flows = {
  'getting-started': { title: '從原始碼到程序', nodes: ['main.rs', 'rustc 與連結器', '執行檔', '作業系統載入', 'CPU 執行'], caption: 'Cargo 負責安排建構。編譯成功之後，作業系統可再次載入同一個執行檔。' },
  'generics-traits-lifetimes': { title: '回傳借用的有效範圍', nodes: ['呼叫端擁有 x 與 y', 'x、y 都可借用的區間', "longest 回傳 &'a str", '最後一次使用回傳值'], caption: "相同的 'a 讓呼叫端只能在兩個輸入都有效的區間內使用結果。標註不會延長任何資料的存活時間。" },
  'smart-pointers': { title: 'Rc 共享配置的釋放時機', nodes: ['Rc::new：strong = 1', 'Rc::clone：strong = 2', '第一個擁有者 drop：strong = 1', '最後一個擁有者 drop：strong = 0'], caption: '強引用計數歸零時銷毀內部值。若仍有 Weak，管理計數的配置會等最後一個 Weak 釋放後才回收。' },
  'concurrency': { title: '通道傳遞所有權', nodes: ['發送端擁有 String', 'tx.send(value) 取得所有權', '通道排隊', 'rx.recv() 取出 String'], caption: '圖中為 send 與 recv 都成功的情況。發送失敗時，SendError<T> 攜回未送出的值。' },
  'async': { title: 'Future 從等待到完成', nodes: ['executor 呼叫 poll', 'Pending：等待事件', 'Waker 通知可再排程', 'executor 再次 poll', 'Ready：取得輸出'], caption: '首次 poll 也可能直接回傳 Ready。wake 只通知執行器應安排後續 poll，並不保證下一次就完成。' },
  'web-server': { title: '一個 HTTP 請求如何經過執行緒池', nodes: ['TcpListener 接受連線', '把工作放入佇列', '工作執行緒取走工作', '讀取請求並寫回回應'], caption: '執行緒池限制同時工作的執行緒數。佇列上限、連線逾時與完整 HTTP 解析是另外的設計工作。' },
};

export function conceptFlow(slug) {
  const flow = flows[slug];
  if (!flow) return '';
  return `<figure class="concept-flow"><figcaption>${flow.title}</figcaption><ol>${flow.nodes.map((node, index) => `<li>${index ? icon('chevron', 16) : ''}<span>${escape(node)}</span></li>`).join('')}</ol><p>${escape(flow.caption)}</p></figure>`;
}

const q = (prompt, choices, answer, explanation) => ({ prompt, choices, answer, explanation });
export const quizzes = {
  'getting-started': q('修改 main.rs 後直接執行舊的 ./main，終端機會顯示什麼？', ['舊的輸出', '新的輸出', '會自動重新編譯'], 0, '作業系統執行的是上次建構產生的二進位檔。重新執行 rustc 或 cargo run，才能讓修改反映到輸出。'),
  'guessing-game': q('使用 read_line 讀到「42」及換行時，怎麼準備給 parse::<u32>()？', ['先用比較運算子', '先用 trim() 去掉首尾空白', '先轉成 &mut u32'], 1, 'read_line 保留換行。trim 回傳去掉首尾空白的切片，parse 再按指定型別解析。'),
  fundamentals: q('區塊最後的運算式加上分號，會帶來什麼變化？', ['區塊仍回傳原值', 'Rust 一律拒絕分號', '這一行成為敘述，區塊在此例得到 ()'], 2, '若區塊沒有尾端運算式，區塊值就是 ()。函式若宣告回傳 i32，這個差異便會成為型別不符。'),
  ownership: q('執行 let b = a; 後，a 原本持有的 String 緩衝區發生了什麼？', ['位元組被複製成兩份', '仍是同一個配置，由 b 負責釋放', '在賦值時就被釋放'], 1, 'String 不實作 Copy。賦值把所有權交給 b，使 a 不可再用。clone 才會讓兩個非空 String 各自持有內容。'),
  structs: q('方法只需讀取欄位，哪種 receiver 能保留呼叫端的所有權？', ['self', '&self', 'Box<Self>'], 1, '&self 借用實例，呼叫端仍持有值。self 與 Box<Self> 以值接收，通常會消耗該實例或盒子。'),
  enums: q('Option<T> 越過 match 時，編譯器需要你處理哪些可能值？', ['只有 Some(T)', '只有 None', '涵蓋 Some(T) 與 None'], 2, 'match 必須窮盡所有可能的值。可以明確寫兩臂，也可以用其餘分支匹配尚未涵蓋的情況。'),
  modules: q('在父模組外存取 child::run 時，何者需要可見？', ['可以到達 child 的路徑，以及 run 本身', '只要檔名相同', '只要 run 編譯通過'], 0, '路徑各節點都須滿足可見性。可用公用模組路徑或 pub use 輸出一條公開路徑。'),
  collections: q('為何不能用 s[1] 取得 String 的第二個字元？', ['所有 UTF-8 字元都固定四位元組', '任意位元組位置不一定對應字元邊界', 'String 無法讀取'], 1, 'UTF-8 字元的位元組長度可變。chars() 走訪 Unicode scalar values，使用者感知字素可能需要專門的分割工具。'),
  errors: q('在返回 Result 的函式中，遇到 Err 時 ? 會做什麼？', ['繼續用預設值', '一律 panic', '提早返回錯誤，必要時進行相容的型別轉換'], 2, '對 Result 使用 ? 時，Ok 取出內容，Err 走提早返回路徑。呼叫端可決定顯示訊息、重試或結束。'),
  'generics-traits-lifetimes': q("標註 fn longest<'a>(x: &'a str, y: &'a str) -> &'a str 有何作用？", ['延長 x 與 y 的生命', '限制回傳引用在兩個借用皆有效的範圍', '保證回傳 x'], 1, '函式可能回傳任一輸入。相同的生命週期參數描述輸入與輸出的關係，呼叫端無法把結果用得比該共同有效範圍更久。'),
  testing: q('cargo test 通過，能直接推得什麼？', ['已執行的測試符合各自斷言', '所有輸入都正確', '正式部署已驗證'], 0, '測試結果只涵蓋執行過的情境，仍要思考測試有沒有包含邊界條件與錯誤路徑。'),
  cli: q('CLI 搜尋工具應把使用錯誤寫去哪裡，讓標準輸出仍可接管線？', ['標準輸出', '標準錯誤', '不輸出'], 1, 'stderr 可顯示診斷，stdout 保留搜尋結果。程序另外回傳非零退出碼，讓呼叫端辨識失敗。'),
  iterators: q('只建立 values.iter().map(f)，沒有消耗迭代器時，f 會如何？', ['立刻對所有元素執行', '先對第一個元素執行', '通常還沒有執行'], 2, 'map 產生惰性轉接器。下一次要求元素時才會呼叫 f，可以用 next、for、collect 等方式驅動。'),
  cargo: q('Cargo.toml 與 Cargo.lock 分別記錄什麼？', ['允許的依賴條件與已解析的具體版本', '兩者都只記錄 Edition', '實際版本與 CPU 型號'], 0, 'manifest 宣告需求，lockfile 釘住解析結果。應用程式在 CI 用 --locked 可阻止無意改動解析結果。'),
  'smart-pointers': q('兩個 Rc 互相以強引用持有對方，外部 Rc 都離開作用域後會怎樣？', ['循環自動被找出並回收', '兩邊強計數可能大於零，值留在配置中', '必然編譯失敗'], 1, '引用計數沒有追蹤式 GC 的循環偵測。把非擁有關係改成 Weak 可打破強引用循環。'),
  concurrency: q('Arc<T> 讓引用計數可跨執行緒，是否也能自動讓任意 T 的寫入安全？', ['可以，Arc 會鎖住 T', '只要 T 有 Debug 就可以', '需要符合 Send/Sync 且用合適方式協調寫入'], 2, 'Arc 用原子操作協調所有權計數，不幫內部資料加鎖。共享可變資料常用 Arc<Mutex<T>>，簡單計數也可評估原子型別。'),
  async: q('Future 回傳 Poll::Pending 之後，Waker 負責什麼？', ['通知執行器安排再 poll', '把工作搬到新執行緒', '回傳 Future 的結果'], 0, 'Pending 表示暫無結果。事件到達時應安排喚醒，執行器稍後再次 poll，直到取得 Ready 或 Future 被丟棄。'),
  'trait-objects': q('使用 &dyn Trait 時，實作的方法通常怎麼選定？', ['依每次建構產生一份泛型特化', '透過 vtable 在執行時選定', '由欄位名稱猜測'], 1, 'Trait object 的指標含資料指標與中繼資料。動態派發可處理異質實作，適合需要執行時多型的場合。'),
  patterns: q('為何 if let 可以使用 Some(x)，而普通 let Some(x) = value 可能無法編譯？', ['if let 有匹配失敗路徑', 'if let 建立新的 enum', 'let 不支援解構'], 0, 'Some(x) 是可反駁模式，當值為 None 時匹配失敗。普通 let 需要不可反駁模式，let-else 則可明確處理失敗。'),
  advanced: q('unsafe 區塊中解參考原始指標之前，誰需要確認指標有效且符合別名規則？', ['借用檢查器會驗證全部前提', '程式作者或提供安全抽象的人', '作業系統一律會修正'], 1, 'unsafe 允許特定操作，程式作者仍需建立安全前提。安全包裝的 API 應讓呼叫者無法破壞內部不變條件。'),
  'web-server': q('一筆 TCP read 是否保證取得完整的 HTTP 請求？', ['是，每次 read 剛好一筆請求', '只有請求超過 MTU 才可能分段', '不保證，TCP 是位元組串流'], 2, 'read 可回傳少於緩衝區長度的資料。HTTP 實作需依協定判斷標頭和訊息內容的邊界，並處理 EOF 與逾時。'),
};

export function quiz(slug) {
  const question = quizzes[slug];
  if (!question) return '';
  return `<section class="quiz" data-quiz="${slug}">
    <p class="lab-kicker">${icon('target', 18)} 自我測驗</p>
    <h2 id="quick-check" tabindex="-1">確認理解</h2>
    <form><fieldset><legend>${escape(question.prompt)}</legend>
    ${question.choices.map((choice, index) => `<label><input type="radio" name="answer" value="${index}" required><span>${escape(choice)}</span></label>`).join('')}
    </fieldset><button type="submit" class="button primary" hidden>檢查答案</button>
    <p data-quiz-feedback role="status"></p></form>
    <details class="answer"><summary>查看完整解析</summary><p><strong>${escape(question.choices[question.answer])}</strong></p><p>${escape(question.explanation)}</p></details>
  </section>`;
}
