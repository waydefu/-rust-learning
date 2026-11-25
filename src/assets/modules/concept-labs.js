import { page } from './context.js';
import { motionFeedback } from './motion.js';

// Small, deterministic conceptual models, not an in-browser Rust compiler.
// The chapter prose remains usable when JavaScript is unavailable.
export function initConceptLabs() {
  const prose = document.querySelector('.prose');
  const models = {
    '08': {title:'Vec：容量用完之後會怎樣？', intro:'這個模型假設容量按 2、4、8 成長，方便觀察重新配置；Rust 不保證實際採用這種成長策略。', mount:0, id:'vec'},
    '13': {title:'迭代器：建立管線與消費管線有何不同？', intro:'此模型用 [1, 2, 3, 4] 示範 filter 與 map 的惰性執行；collect 才會驅動迭代器完成工作。', mount:1, id:'iterator'},
    '15': {title:'Rc：最後一個擁有者離開時發生什麼？', intro:'模擬單執行緒 Rc 的強引用計數；clone 新增共享擁有者而非複製 String。此模型不包含 Weak 或循環引用。', mount:5, id:'rc'},
    '16': {title:'兩個執行緒，為什麼會遺失一次加法？', intro:'模擬兩次「讀取、加一、寫回」交錯；這是抽象的邏輯競態模型，不是在 Rust 中使用未同步的可變記憶體。', mount:4, id:'race'},
    '17': {title:'Future 的三個時刻：輪詢、喚醒、完成', intro:'模擬一個必須等待 I/O 的 Future。喚醒只是要求重新排程，不保證每種 Future 下次輪詢一定完成。', mount:2, id:'future'},
  };
  const model = models[page.chapter];
  if (!prose || !model) return;
  const element = (tag, text, className) => {
    const el = document.createElement(tag);
    if (text) el.textContent = text;
    if (className) el.className = className;
    return el;
  };
  const panel = element('section', '', 'concept-lab');
  panel.dataset.conceptLab = model.id;
  const heading = element('h3', model.title);
  heading.id = 'concept-lab-' + model.id;
  panel.setAttribute('aria-labelledby',heading.id);
  const explanation = element('p',model.intro);
  const view = element('div','','concept-view');
  view.setAttribute('aria-hidden','true');
  const controls = element('div','','concept-controls');
  const status = element('p','','concept-status');
  status.setAttribute('role','status');
  status.setAttribute('aria-atomic','true');
  const button = (label, action) => {
    const b = element('button',label,'concept-button');
    b.type = 'button';
    b.addEventListener('click',action);
    controls.append(b);
    return b;
  };
  const cells = (values, occupied) => {
    view.replaceChildren(...values.map((value,i) => {
      const cell = element('span',String(value),'concept-cell');
      if (occupied(i)) cell.dataset.filled = 'true';
      return cell;
    }));
    motionFeedback(view,[{transform:'translateY(2px)'},{transform:'none'}],180);
  };
  panel.append(heading,explanation,view,controls,status);
  const sections = [...prose.querySelectorAll(':scope > h2')];
  const next = sections[model.mount + 1];
  if (next) next.before(panel); else prose.append(panel);
  if (model.id === 'vec') {
    let length=0, capacity=2;
    const render = message => {
      cells(Array.from({length:capacity},(_,i)=>i<length?i+1:'—'),i=>i<length);
      push.disabled = length===8;
      pop.disabled = length===0;
      status.textContent = 'len = '+length+'，capacity = '+capacity+'。'+message;
    };
    const push = button('push 一個元素', () => {
      const grow = length===capacity;
      if (grow) capacity*=2;
      length++;
      render(grow?'原容量不足，模型配置了更大的緩衝區並搬移舊元素。':'仍有空位，不必擴容。');
    });
    const pop = button('pop 一個元素',()=>{if(length) length--;render('移除末端元素；pop 不會自動縮小容量。');});
    button('重設',()=>{length=0;capacity=2;render('初始緩衝區。');});
    render('初始緩衝區。');
  } else if (model.id === 'iterator') {
    let stage=0;
    const render=()=>{
      const messages=[
        '原始資料 [1, 2, 3, 4]；尚未建立轉接器。',
        '已建立 filter：保留偶數。尚未走訪元素。',
        '已建立 map：乘以 10。仍未走訪元素。',
        'collect 驅動管線：1、3 被過濾，2、4 乘以 10，得到 [20, 40]。'
      ];
      cells(stage===3?[20,40]:[1,2,3,4],()=>stage===3);
      filter.disabled=stage!==0; map.disabled=stage!==1; collect.disabled=stage!==2;
      status.textContent=messages[stage];
    };
    const filter=button('建立 filter',()=>{stage=1;render();});
    const map=button('建立 map',()=>{stage=2;render();});
    const collect=button('collect()',()=>{stage=3;render();});
    button('重設',()=>{stage=0;render();});
    render();
  } else if (model.id === 'rc') {
    let owners=1;
    const render=message=>{
      cells(['強引用 '+owners,owners?'String: shared':'內容已釋放'],i=>i===0||owners>0);
      clone.disabled=owners===0||owners===4;
      drop.disabled=owners===0;
      status.textContent='strong_count = '+owners+'。'+message;
    };
    const clone=button('Rc::clone',()=>{owners++;render('新增一個擁有者，沒有複製 String 內容。');});
    const drop=button('drop 一個 Rc',()=>{owners--;render(owners?'仍有擁有者，保留資料。':'最後一個強擁有者離開，內容被清理。');});
    button('重設',()=>{owners=1;render('建立一個 Rc 擁有者。');});
    render('建立一個 Rc 擁有者。');
  } else if (model.id === 'race') {
    let step=0, locked=false, total=0, a=null, b=null;
    const render=message=>{
      cells(['共享值 '+total,'A '+(a??'—'),'B '+(b??'—')],()=>true);
      advance.disabled=step===4;
      status.textContent='共享值 '+total+'；A 讀值 '+(a??'尚未讀取')+'；B 讀值 '+(b??'尚未讀取')+'。'+message;
    };
    const reset=()=>{step=0;total=0;a=null;b=null;render(locked?'Mutex 版本：每次完整更新都持有鎖。':'無互斥版本：兩個讀取可能交錯。');};
    const advance=button('執行下一步',()=>{
      let message='';
      if(locked){
        if(step===0){a=total;message='A 取得鎖並讀取 '+a;}
        if(step===1){total=a+1;message='A 寫回 '+total+' 並釋放鎖。';}
        if(step===2){b=total;message='B 取得鎖並讀取 '+b;}
        if(step===3){total=b+1;message='B 寫回 '+total+'，結果為 2。';}
      }else{
        if(step===0){a=total;message='A 讀取 '+a;}
        if(step===1){b=total;message='B 也讀取 '+b;}
        if(step===2){total=a+1;message='A 寫回 '+total;}
        if(step===3){total=b+1;message='B 根據舊值寫回 '+total+'，一次更新遺失。';}
      }
      step++;render(message);
    });
    const toggle=button('改用 Mutex',()=>{locked=!locked;toggle.textContent=locked?'改用無互斥模型':'改用 Mutex';reset();});
    button('重設',reset);reset();
  } else {
    let phase=0;
    const messages=['已建立 Future，函式尚未開始。','poll 回傳 Pending，等待 I/O 並登記 Waker。','I/O 準備好，Waker 通知 executor 重新排程。','下一次 poll 回傳 Ready(42)，此模型的工作完成。'];
    const render=()=>{
      cells(['建立','Pending','喚醒','Ready'],i=>i===phase);
      poll.disabled=phase===1||phase===3;
      wake.disabled=phase!==1;
      status.textContent=messages[phase];
    };
    const poll=button('poll()',()=>{phase=phase===0?1:3;render();});
    const wake=button('模擬 I/O 喚醒',()=>{phase=2;render();});
    button('重設',()=>{phase=0;render();});
    render();
  }
}
