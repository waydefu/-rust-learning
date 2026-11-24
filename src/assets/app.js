const page = JSON.parse(document.querySelector('#page-data').textContent);
const chapterPaths = new Map(page.chapterPaths);
const storage = {
  get(key) {
    try { return localStorage.getItem(`rust-learning-${key}`); } catch { return null; }
  },
  set(key, value) {
    try { localStorage.setItem(`rust-learning-${key}`, value); return true; } catch { return false; }
  },
};

const root = document.documentElement;
const motionPreference = matchMedia('(prefers-reduced-motion: reduce)');
const feedbackAnimations = new WeakMap();
const activeAnimations = new Set();
function motionFeedback(element, frames, duration = 220) {
  if (!element || motionPreference.matches || typeof element.animate !== 'function') return;
  feedbackAnimations.get(element)?.cancel();
  const animation = element.animate(frames, {duration, easing: 'cubic-bezier(.2,.75,.25,1)'});
  feedbackAnimations.set(element, animation);
  activeAnimations.add(animation);
  const cleanup = () => activeAnimations.delete(animation);
  animation.addEventListener('finish', cleanup, {once:true});
  animation.addEventListener('cancel', cleanup, {once:true});
}
motionPreference.addEventListener('change', event => {
  if (event.matches) for (const animation of activeAnimations) animation.cancel();
  if (event.matches) document.querySelector('.startup-motion')?.remove();
});
const preference = matchMedia('(prefers-color-scheme: dark)');
const themeButton = document.querySelector('#theme-toggle');
function updateThemeButton() {
  const isDark = root.dataset.theme ? root.dataset.theme === 'dark' : preference.matches;
  themeButton.setAttribute('aria-label', isDark ? '切換淺色模式' : '切換深色模式');
  themeButton.title = themeButton.getAttribute('aria-label');
}
themeButton.hidden = false;
updateThemeButton();
themeButton.addEventListener('click', () => {
  const isDark = root.dataset.theme ? root.dataset.theme === 'dark' : preference.matches;
  root.dataset.theme = isDark ? 'light' : 'dark';
  storage.set('theme', root.dataset.theme);
  updateThemeButton();
});
preference.addEventListener('change', updateThemeButton);

const menu = document.querySelector('#menu-dialog');
const menuOpen = document.querySelector('#menu-open');
menuOpen.hidden = false;
menuOpen.addEventListener('click', () => menu.showModal());
matchMedia('(min-width: 811px)').addEventListener('change', event => {
  if (event.matches && menu.open) menu.close();
});

const search = document.querySelector('#search-dialog');
const searchButton = document.querySelector('#search-open');
const searchInput = document.querySelector('#search-input');
const searchStatus = document.querySelector('#search-status');
const searchResults = document.querySelector('#search-results');
const shortcut = document.querySelector('[data-shortcut]');
shortcut.textContent = /Mac|iPhone|iPad/.test(navigator.platform) ? '⌘ K' : 'Ctrl K';
searchButton.closest(".search-wrap").hidden = false;
let indexRequest;
let searchRevision = 0;
const normalize = value => value.normalize('NFKC').toLowerCase();
function loadIndex() {
  indexRequest ??= fetch(`${page.base}assets/search-index.json`).then(response => {
    if (!response.ok) throw new Error('無法下載搜尋索引');
    return response.json();
  }).catch(error => {
    indexRequest = undefined;
    throw error;
  });
  return indexRequest;
}

function openSearch() {
  if (menu.open) menu.close();
  if (!search.open) search.showModal();
  searchInput.focus();
  renderSearch();
}
searchButton.addEventListener('click', openSearch);
search.addEventListener('click', event => {
  if (event.target !== search) return;
  const bounds = search.getBoundingClientRect();
  if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) search.close();
});
document.addEventListener('keydown', event => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault();
    openSearch();
  }
});
searchInput.addEventListener('input', renderSearch);
searchInput.addEventListener('keydown', event => {
  if (event.key !== 'ArrowDown') return;
  const first = searchResults.querySelector('a');
  if (first) {
    event.preventDefault();
    first.focus();
  }
});
searchResults.addEventListener('keydown', event => {
  if (!['ArrowUp', 'ArrowDown'].includes(event.key) || !event.target.matches('a')) return;
  const links = Array.from(searchResults.querySelectorAll('a'));
  const next = links[links.indexOf(event.target) + (event.key === 'ArrowUp' ? -1 : 1)];
  if (next || event.key === 'ArrowUp') {
    event.preventDefault();
    (next ?? searchInput).focus();
  }
});

async function renderSearch() {
  const revision = ++searchRevision;
  const query = normalize(searchInput.value.trim());
  searchResults.replaceChildren();
  if (!query) {
    searchStatus.textContent = '輸入章名、概念、程式碼或編譯錯誤。';
    return;
  }
  searchStatus.textContent = '搜尋中…';
  try {
    const entries = await loadIndex();
    if (revision !== searchRevision) return;
    const terms = query.split(/\s+/).filter(Boolean);
    const found = entries.flatMap(entry => {
      const title = normalize(entry.title);
      const body = normalize(`${entry.id} ${entry.description} ${entry.text}`);
      const heading = entry.headings.find(section => terms.every(term => normalize(section.label).includes(term)));
      if (!terms.every(term => title.includes(term) || body.includes(term))) return [];
      const score = title.includes(query) ? 4 : heading ? 3 : body.includes(query) ? 2 : 1;
      return [{ entry, score, heading: score === 3 ? heading : null }];
    }).sort((a, b) => b.score - a.score);
    searchStatus.textContent = found.length ? `找到 ${found.length} 筆${found.length > 12 ? '，顯示前 12 筆' : ''}` : '沒有符合的結果，試試較短的詞。';
    for (const { entry, heading } of found.slice(0, 12)) {
      const li = document.createElement('li');
      const link = document.createElement('a');
      link.href = entry.href + (heading ? '#' + encodeURIComponent(heading.id) : '');
      const title = document.createElement('strong');
      title.textContent = /^\d+$/.test(entry.id) ? `${entry.id}　${entry.title}` : entry.title;
      const description = document.createElement('small');
      description.textContent = heading?.label ?? entry.description;
      link.append(title, description);
      li.append(link);
      searchResults.append(li);
    }
  } catch {
    if (revision === searchRevision) searchStatus.textContent = '無法載入索引。請再試一次，或從課程導覽開啟教材。';
  }
}

const progress = document.querySelector('[data-progress]');
const complete = document.querySelector('#complete');
const completed = new Set([...chapterPaths.keys()].filter(id => storage.get(`completed-${id}`) === '1'));
if (page.chapter) storage.set('last-chapter', page.chapter);

function renderProgress() {
  document.querySelectorAll('[data-nav-chapter]').forEach(link => {
    link.querySelector('.nav-done').hidden = !completed.has(link.dataset.navChapter);
  });
  document.querySelectorAll('[data-card-chapter]').forEach(card => {
    card.querySelector('.card-done').hidden = !completed.has(card.dataset.cardChapter);
  });
  if (complete) {
    const done = completed.has(page.chapter);
    complete.setAttribute('aria-pressed', String(done));
    complete.querySelector('span').textContent = done ? '取消完成' : '標記完成';
  }
  if (!progress) return;
  progress.querySelector('progress').value = completed.size;
  progress.querySelector('[data-progress-count]').textContent = `${completed.size} / ${chapterPaths.size} 章`;
  progress.querySelector('[data-progress-percent]').textContent = `${Math.round(completed.size * 100 / chapterPaths.size)}%`;
  const lastChapter = storage.get('last-chapter');
  if (chapterPaths.has(lastChapter)) {
    const path = chapterPaths.get(lastChapter);
    const section = storage.get(`last-section-${lastChapter}`);
    const resumeHref = `${page.base}${path}/${section ? '#' + encodeURIComponent(section) : ''}`;
    for (const link of [progress.querySelector('[data-resume]'), document.querySelector('[data-start]')]) {
      link.href = resumeHref;
      link.textContent = link.matches('[data-start]') ? '繼續學習' : `繼續第 ${Number(lastChapter)} 章`;
    }
  }
}
renderProgress();
if (complete) {
  complete.hidden = false;
  complete.addEventListener('click', () => {
    const done = !completed.has(page.chapter);
    const persisted = storage.set(`completed-${page.chapter}`, done ? '1' : '0');
    if (done) completed.add(page.chapter);
    else completed.delete(page.chapter);
    renderProgress();
    document.querySelector('[data-completion-status]').textContent = !persisted
      ? '目前瀏覽器未允許儲存，重新載入後可能遺失進度。'
      : done ? '本章已標記完成，進度已儲存。' : '已取消本章的完成標記。';
  });
}
window.addEventListener('storage', event => {
  if (!event.key?.startsWith('rust-learning-completed-')) return;
  const id = event.key.slice('rust-learning-completed-'.length);
  if (!chapterPaths.has(id)) return;
  if (event.newValue === '1') completed.add(id);
  else completed.delete(id);
  renderProgress();
});

for (const link of document.querySelectorAll('.mobile-toc a[href^="#"]')) {
  link.addEventListener('click', () => link.closest('details').open = false);
}
const tocLinks = Array.from(document.querySelectorAll('.desktop-toc a[href^="#"]'));
const sections = tocLinks.map(link => document.getElementById(link.getAttribute('href').slice(1))).filter(Boolean);
if (sections.length) {
  let active;
  let frame;
  function getHashSection() {
    try {
      const id = decodeURIComponent(location.hash.slice(1));
      return sections.find(section => {
        if (section.id !== id) return false;
        const top = section.getBoundingClientRect().top;
        return top >= 0 && top < innerHeight - 32;
      });
    } catch { return undefined; }
  }
  const updateLocation = () => {
    const pinned = getHashSection();
    const maxScroll = document.documentElement.scrollHeight - innerHeight;
    const atBottom = maxScroll > 0 && scrollY >= maxScroll - 2;
    const underHeader = sections.filter(section => section.getBoundingClientRect().top <= 142).at(-1);
    const next = pinned ?? (atBottom ? sections.at(-1) : underHeader);
    if (next !== active) {
      active = next;
      for (const link of tocLinks) {
        if (link.getAttribute('href') === `#${active?.id}`) link.setAttribute('aria-current', 'location');
        else link.removeAttribute('aria-current');
      }
      if (page.chapter) storage.set(`last-section-${page.chapter}`, active?.id ?? '');
    }
  };
  window.addEventListener('scroll', () => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(updateLocation);
  }, { passive: true });
  window.addEventListener('hashchange', () => requestAnimationFrame(updateLocation));
  window.addEventListener('pageshow', () => requestAnimationFrame(updateLocation));
  updateLocation();
}

for (const button of document.querySelectorAll('[data-copy]')) {
  button.hidden = false;
  let restore;
  button.addEventListener('click', async () => {
    const label = button.querySelector('span');
    const code = button.closest('.code-block').querySelector('code');
    try {
      await navigator.clipboard.writeText(code.textContent.replace(/\n$/, ''));
      label.textContent = '已複製';
      motionFeedback(button, [{transform:'scale(.96)'},{transform:'none'}]);
    } catch {
      const selection = window.getSelection();
      if (!selection) return;
      const range = document.createRange();
      range.selectNodeContents(code);
      selection.removeAllRanges();
      selection.addRange(range);
      label.textContent = '已選取，請複製';
    }
    clearTimeout(restore);
    restore = setTimeout(() => label.textContent = '複製', 2200);
  });
}

const lab = document.querySelector('[data-ownership]');
if (lab) {
  const controls = lab.querySelector('[data-steps]');
  const visual = lab.querySelector('#memory-state');
  controls.hidden = false;
  controls.addEventListener('click', event => {
    const button = event.target.closest('[data-step]');
    if (!button) return;
    const step = page.ownershipSteps[Number(button.dataset.step)];
    for (const control of controls.querySelectorAll('button')) control.setAttribute('aria-pressed', String(control === button));
    visual.dataset.owner = step.owner ?? 'none';
    visual.dataset.borrowed = String(Boolean(step.borrowed));
    lab.querySelector('[data-lab-code]').textContent = step.code;
    lab.querySelector('[data-binding-a]').textContent = step.source;
    lab.querySelector('[data-binding-b]').textContent = step.target;
    lab.querySelector('[data-lab-message]').textContent = step.message;
    const owner = step.owner === 'a' ? '[data-binding-a]' : step.owner === 'b' ? '[data-binding-b]' : null;
    if (owner) motionFeedback(lab.querySelector(owner), [{transform:'translateX(-3px)'},{transform:'none'}]);
    if (step.owner) motionFeedback(lab.querySelector('.memory-bytes'), [{transform:'scale(.97)'},{transform:'none'}], 300);
  });
}

const quiz = document.querySelector('[data-quiz]');
if (quiz) {
  const form = quiz.querySelector('form');
  const feedback = form.querySelector('[data-quiz-feedback]');
  const answer = quiz.querySelector('.answer');
  form.querySelector('button').hidden = false;
  form.addEventListener('change', () => {
    feedback.textContent = '';
    delete feedback.dataset.result;
  });
  form.addEventListener('submit', event => {
    event.preventDefault();
    const chosen = new FormData(form).get('answer');
    if (chosen === null) return;
    const correct = Number(chosen) === page.answer;
    feedback.dataset.result = correct ? 'correct' : 'retry';
    feedback.textContent = correct ? '答對了。下方有完整解析。' : '再試一次，或展開解析確認原因。';
    if (correct) answer.open = true;
    motionFeedback(feedback, [{transform:'translateY(3px)'},{transform:'none'}]);
  });
}


// A non-modal welcome animation once per tab session. No remote resources,
// content hiding, focus changes, or delayed navigation; no-JS pages stay intact.
function startupMotion() {
  if (motionPreference.matches || location.hash || !document.querySelector('.hero')) return;
  try {
    if (sessionStorage.getItem('rust-learning-startup') === '1') return;
    sessionStorage.setItem('rust-learning-startup', '1');
  } catch { return; }
  const badge = document.createElement('div');
  badge.className = 'startup-motion';
  badge.setAttribute('aria-hidden', 'true');
  const logo = document.createElement('img');
  logo.src = page.base + 'assets/favicon.svg';
  logo.alt = '';
  const copy = document.createElement('div');
  const title = document.createElement('strong');
  title.textContent = 'Rust 學習工坊';
  const sub = document.createElement('small');
  sub.textContent = 'RUST LEARNING LAB';
  copy.append(title, sub);
  badge.append(logo, copy);
  document.body.append(badge);
  const remove = () => badge.remove();
  badge.addEventListener('animationend', event => { if (event.target === badge) remove(); }, {once:false});
  setTimeout(remove, 1400);
}
startupMotion();


// Reading progress reflects actual scroll, not chapter completion. Decorative;
// no focus or announcement changes. The content is never hidden.
if (page.chapter) {
  const indicator = document.createElement('div');
  indicator.className = 'reading-indicator';
  indicator.setAttribute('aria-hidden', 'true');
  const fill = document.createElement('span');
  indicator.append(fill);
  document.body.append(indicator);
  let pending = false;
  const update = () => {
    pending = false;
    const range = Math.max(0, root.scrollHeight - innerHeight);
    const fraction = range ? Math.min(1, Math.max(0, scrollY / range)) : 1;
    fill.style.transform = 'scaleX(' + fraction + ')';
  };
  const schedule = () => {
    if (!pending) { pending = true; requestAnimationFrame(update); }
  };
  addEventListener('scroll', schedule, {passive:true});
  addEventListener('resize', schedule, {passive:true});
  addEventListener('pageshow', schedule);
  if ('ResizeObserver' in window) new ResizeObserver(schedule).observe(document.body);
  update();
}


// Small, deterministic conceptual models, not an in-browser Rust compiler.
// The chapter prose remains usable when JavaScript is unavailable.
(() => {
  const prose = document.querySelector('.prose');
  const models = {
    '08': {title:'Vec：容量用完之後會怎樣？', intro:'這個模型假設容量按 2、4、8 成長，方便觀察重新配置；Rust 不保證實際採用這種成長策略。', mount:0, id:'vec'},
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
      status.textContent = 'len = '+length+'，capacity = '+capacity+'。'+message;
    };
    const push = button('push 一個元素', () => {
      const grow = length===capacity;
      if (grow) capacity*=2;
      length++;
      render(grow?'原容量不足，模型配置了更大的緩衝區並搬移舊元素。':'仍有空位，不必擴容。');
    });
    button('pop 一個元素',()=>{if(length) length--;render('移除末端元素；pop 不會自動縮小容量。');});
    button('重設',()=>{length=0;capacity=2;render('初始緩衝區。');});
    render('初始緩衝區。');
  } else if (model.id === 'race') {
    let step=0, locked=false, total=0, a=null, b=null;
    const render=message=>{
      cells(['共享值 '+total,'A '+(a??'—'),'B '+(b??'—')],()=>true);
      advance.disabled=step===4;
      status.textContent=message;
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
})();
