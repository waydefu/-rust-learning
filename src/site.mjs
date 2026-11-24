import { catalog, chapters, resources, stages, escape, renderTokens } from './course.mjs';
import { conceptFlow, ownershipLab, quiz, ownershipSteps, quizzes } from './learning-labs.mjs';
import { icon } from './icons.mjs';

export const repository = 'https://github.com/waydefu/-rust-learning';
export const siteName = 'Rust 學習工坊';

const safeJSON = data => JSON.stringify(data).replaceAll('<', '\\u003c');

export function makeSite(base, {nativeLayout = false} = {}) {
  const url = (path = '') => path ? `${base}${path}/` : base;
  const asset = name => `${base}assets/${name}`;
  const chapterName = entry => entry.id.startsWith('A') ? entry.id : `第 ${Number(entry.id)} 章`;
  const navRow = (target, label, name, current, meta = '') => `<a href="${url(target)}" class="nav-row"${current === target ? ' aria-current="page"' : ''}>${icon(name, 19)}<span>${label}</span>${meta ? `<small>${meta}</small>` : ''}</a>`;

  function navigation(current, id) {
    return `<nav class="sidebar-inner" aria-label="課程導覽">
      <p class="nav-caption">學習</p>
      ${navRow('', '課程總覽', 'home', current)}
      ${navRow('guide', '學習指南', 'route', current)}
      ${navRow('resources', '延伸資源', 'layers', current, resources.length)}
      <p class="nav-caption grouped-caption">章節目錄</p>
      ${stages.map((stage, index) => `<details class="nav-group"${index === 0 && current === '' || stage.chapters.some(entry => entry.id === id) ? ' open' : ''}>
        <summary><span class="stage-dot stage-${stage.id}"></span><span>${stage.title}</span>${icon('down', 16)}</summary>
        <div class="nav-chapters">${stage.chapters.map(entry => `<a href="${url(entry.path)}"${entry.id === id ? ' aria-current="page"' : ''} data-nav-chapter="${entry.id}"><span class="nav-number">${entry.id}</span><span>${entry.title}</span><span class="nav-done" aria-label="已完成" hidden>${icon('check', 14)}</span></a>`).join('')}</div>
      </details>`).join('')}
      <div class="sidebar-foot"><span class="stage-dot"></span> Rust 2024 Edition</div>
    </nav>`;
  }

  function layout({current, id = '', title, description, body, data = {}}) {
    if (nativeLayout) return {
      current, id, title, description, navigation: navigation(current,id), body,
      pageData: safeJSON({base, chapterPaths: chapters.map(entry => [entry.id, entry.path]), ...data}),
    };
    return `<!doctype html>
<html lang="zh-Hant">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="description" content="${escape(description)}">
<meta name="color-scheme" content="light dark">
<title>${escape(title === siteName ? title : `${title}｜${siteName}`)}</title>
<link rel="icon" href="${asset('favicon.svg')}" type="image/svg+xml">
<script src="${asset('theme.js')}"></script>
<link rel="stylesheet" href="${asset('site.css')}">
<script type="module" src="${asset('app.js')}"></script>
</head>
<body>
<a class="skip-link" href="#main">跳至主要內容</a>
<header class="site-header">
  <button type="button" id="menu-open" class="icon-button mobile-menu" hidden aria-label="開啟課程導覽" aria-haspopup="dialog">${icon('menu')}</button>
  <a class="brand" href="${url()}"><img src="${asset('favicon.svg')}" width="36" height="36" alt=""><span>${siteName}<small>RUST LEARNING LAB</small></span></a>
  <div class="header-actions">
    <div class="search-wrap" hidden><button type="button" class="search-button" id="search-open" aria-haspopup="dialog" aria-label="搜尋教材">${icon('search', 19)}<span>搜尋教材</span></button><kbd data-shortcut aria-hidden="true">⌘ K</kbd></div>
    <span class="header-divider"></span>
    <button type="button" class="icon-button theme-button" id="theme-toggle" aria-label="切換深色模式" hidden><span class="light-icon">${icon('sun')}</span><span class="dark-icon">${icon('moon')}</span></button>
    <a class="icon-button github-link" href="${repository}" aria-label="GitHub 儲存庫">${icon('github')}</a>
  </div>
</header>
<aside class="sidebar">${navigation(current,id)}</aside>
<dialog class="mobile-drawer" id="menu-dialog" aria-labelledby="menu-title"><div class="dialog-head"><strong id="menu-title">課程導覽</strong><form method="dialog"><button class="icon-button" aria-label="關閉導覽">${icon('close')}</button></form></div>${navigation(current,id)}</dialog>
<dialog class="search-dialog" id="search-dialog" aria-labelledby="search-title">
  <div class="dialog-head"><strong id="search-title">搜尋教材</strong><form method="dialog"><button class="icon-button" aria-label="關閉搜尋">${icon('close')}</button></form></div>
  <label class="search-field">${icon('search')}<span class="sr-only">搜尋詞</span><input type="search" id="search-input" autocomplete="off" placeholder="例如所有權、Future 或 E0382"></label>
  <p id="search-status" role="status"></p><ul id="search-results"></ul>
</dialog>
<main id="main">${body}</main>
<script type="application/json" id="page-data">${safeJSON({base, chapterPaths: chapters.map(entry => [entry.id, entry.path]), ...data})}</script>
</body>
</html>\n`;
  }

  const cta = (path, label, attributes = '') => `<a class="button primary" href="${url(path)}" ${attributes}>${label}</a>`;
  const footer = () => `<footer class="site-footer"><span>Rust 學習工坊 <span aria-hidden="true">·</span> 繁體中文</span><a href="${url('guide')}">學習指南</a><a href="${repository}">GitHub</a></footer>`;
  const chapterCard = (entry, stage, featured = false) => `<a class="chapter-card${featured ? ' feature-card' : ''}" href="${url(entry.path)}" data-card-chapter="${entry.id}">
    <div class="card-top"><span class="card-number stage-${stage.id}">${entry.id}</span>${featured ? '<span class="interactive-tag">互動圖解</span>' : ''}<span class="card-done" hidden>${icon('check', 15)} 已完成</span></div>
    <h3>${entry.title}</h3><p>${entry.description}</p>
    <span class="card-link">閱讀章節 ${icon('chevron', 15)}</span>
  </a>`;

  function home() {
    const first = chapters[0];
    const body = `<div class="home-page">
    <div class="breadcrumb"><span>學習</span>${icon('chevron', 14)}<span aria-current="page">課程總覽</span></div>
    <section class="hero" aria-labelledby="hero-title">
      <div class="hero-copy">
        <div class="edition"><span></span> RUST 2024 EDITION</div>
        <h1 id="hero-title">從第一行程式，<br><em>到真正理解 Rust。</em></h1>
        <p>不必有程式設計經驗。從電腦如何執行第一行程式開始，跟著範例弄懂資料如何保存、程式為何出錯，再一步步寫出自己的工具。</p>
        <div class="hero-actions">${cta(first.path,'開始學習','data-start')}<a class="button hero-secondary" href="#learning-path">${icon('route', 18)} 課程路線</a></div>
        <div class="hero-meta"><span>${icon('book', 16)} ${chapters.length} 章課程</span><span>${icon('layers', 16)} ${resources.length} 份延伸教材</span></div>
      </div>
      <div class="hero-figure"><img src="${asset('ownership.svg')}" width="448" height="296" alt="所有權移動示意。a 已移動，b 指向原本的四個堆積位元組。"></div>
    </section>
    <section class="progress-card" data-progress aria-labelledby="progress-title">
      <span class="progress-icon">${icon('target', 24)}</span>
      <div class="progress-copy"><h2 id="progress-title">我的學習進度</h2><p data-progress-description>讀完教材並試做練習後，可以在這裡記下學習進度。</p></div>
      <div class="progress-meter"><div><span data-progress-count>0 / ${chapters.length} 章</span><strong data-progress-percent>0%</strong></div><progress value="0" max="${chapters.length}" aria-label="課程完成進度"></progress></div>
      <a class="button outline resume-button" href="${url(first.path)}" data-resume>前往第 1 章</a>
    </section>
    <section class="learning-path" id="learning-path" aria-labelledby="path-title">
      <div class="section-head"><div><h2 id="path-title">學習路線</h2><p>由語法和記憶體打底，逐步走向系統實作。</p></div><a class="button outline" href="${url('guide/outline')}">${icon('list',17)} 完整目錄</a></div>
      ${stages.map((stage, index) => `<section class="stage-section" id="stage-${stage.id}">
        <div class="stage-heading"><span class="stage-icon stage-${stage.id}">${icon(stage.icon,22)}</span><div><h3>${stage.title}</h3><p>${stage.description}</p></div><span class="stage-range">第 ${Number(stage.chapters[0].id)}–${Number(stage.chapters.at(-1).id)} 章</span></div>
        <div class="chapter-grid">${stage.chapters.map(entry => chapterCard(entry, stage, entry.slug === 'ownership')).join('')}</div>
      </section>`).join('')}
    </section>
    ${footer()}</div>`;
    return layout({current:'', title:siteName, description:'繁體中文 Rust 2024 學習平台。從編譯、所有權與借用一路到非同步和網頁伺服器。',body});
  }

  function resourceIndex() {
    const body = `<div class="home-page secondary-page">
      <div class="breadcrumb"><a href="${url()}">課程總覽</a>${icon('chevron',14)}<span>延伸資源</span></div>
      <div class="section-head"><div><h1>延伸資源</h1><p>查找底層機制、開發環境與編譯診斷，或用題庫驗收理解。</p></div></div>
      <div class="chapter-grid resource-grid">${resources.map(entry => `<a href="${url(entry.path)}" class="chapter-card"><div class="card-top"><span class="card-number stage-engineering">${entry.id}</span></div><h2>${entry.title}</h2><p>${entry.description}</p><span class="card-link">閱讀教材 ${icon('chevron',15)}</span></a>`).join('')}</div>
      ${footer()}
    </div>`;
    return layout({current:'resources', title:'延伸資源', description:'八份 Rust 延伸教材與推理題庫。',body});
  }

  function lesson({entry, title, tokens, headings, nativeBody}) {
    const index = chapters.indexOf(entry);
    const isChapter = index !== -1;
    const isResource = resources.includes(entry);
    const stage = stages.find(item => item.id === entry.group);
    const label = isChapter ? chapterName(entry) : isResource ? `延伸教材 ${entry.id}` : '課程導覽';
    const previous = isChapter ? chapters[index - 1] : null;
    const next = isChapter ? chapters[index + 1] : null;
    const flow = conceptFlow(entry.slug);
    const isOwnership = entry.slug === 'ownership';
    const firstSection = headings[0]?.tokenIndex - 3;
    const firstPart = flow ? (nativeBody?.intro ?? renderTokens(tokens.slice(0, firstSection))) : '';
    const rest = nativeBody
      ? (flow ? nativeBody.rest : nativeBody.intro + nativeBody.rest)
      : (flow ? renderTokens(tokens.slice(firstSection)) : renderTokens(tokens));
    const outline = `<ul>${isOwnership ? '<li><a href="#ownership-lab">互動記憶體圖</a></li>' : ''}${headings.filter(heading => heading.level === 2).map(heading => `<li><a href="#${escape(heading.id)}">${escape(heading.label)}</a></li>`).join('')}${isChapter ? '<li><a href="#quick-check">自我測驗</a></li>' : ''}</ul>`;
    const isOutline = entry.slug === 'outline';
    const outlineLinks = isOutline ? `<section class="outline-links"><h2>網站章節索引</h2>${stages.map(item => `<h3>${item.title}</h3><ol>${item.chapters.map(chapter => `<li><a href="${url(chapter.path)}">${chapterName(chapter)}　${chapter.title}</a></li>`).join('')}</ol>`).join('')}<p><a href="${url('resources')}">瀏覽延伸教材</a></p></section>` : '';
    const sourceURL = `https://docs.google.com/document/d/${entry.source.id}/edit`;
    const extra = entry.slug === 'guide' ? `<p class="guide-links"><a href="${url('guide/outline')}">課程總目錄</a><a href="${url('resources')}">延伸資源</a></p>` : '';
    const body = `<div class="reader-page">
      <div class="breadcrumb"><a href="${url()}">課程總覽</a>${icon('chevron',14)}${isResource ? `<a href="${url('resources')}">延伸資源</a>${icon('chevron',14)}` : ''}<span>${entry.title}</span></div>
      <div class="reader-grid"><article class="reader">
        <header class="lesson-head"><div class="lesson-label">${icon(stage?.icon ?? 'book',17)} <span>${label}${stage ? `<span aria-hidden="true"> · </span>${stage.title}` : ''}</span></div>
        <h1>${escape(title)}</h1><p>${entry.description}</p></header>
        <details class="mobile-toc"><summary>${icon('list',17)} 本章目錄</summary><nav aria-label="本章目錄（手機）">${outline}</nav></details>
        <div class="prose">${extra}${outlineLinks}${isOwnership ? ownershipLab() : ''}${firstPart}${flow}${rest}</div>
        ${quiz(entry.slug)}
        ${isChapter ? `<section class="completion" aria-labelledby="complete-title"><div><h2 id="complete-title">完成本章</h2><p data-completion-status role="status">試著用自己的話說明本章重點，再記下閱讀進度。</p></div><button type="button" class="button outline" id="complete" aria-pressed="false" hidden>${icon('check',17)} <span>標記完成</span></button></section>` : ''}
        <p class="source-link">教材來源 <a href="${sourceURL}">Google Drive 原始文件</a></p>
        ${isChapter ? `<nav class="chapter-pagination" aria-label="相鄰章節">
        ${previous ? `<a href="${url(previous.path)}"><small>上一章</small><strong>${previous.title}</strong></a>` : `<a href="${url('guide')}"><small>開始之前</small><strong>學習指南</strong></a>`}
        ${next ? `<a href="${url(next.path)}"><small>下一章</small><strong>${next.title}</strong></a>` : `<a href="${url('resources')}"><small>繼續學習</small><strong>延伸資源</strong></a>`}</nav>` : ''}
      </article><aside class="desktop-toc"><nav aria-label="本章目錄"><h2>本章目錄</h2>${outline}</nav></aside></div>
      ${footer()}
    </div>`;
    const spec = layout({current:entry.path,id:entry.id,title:entry.title,description:entry.description,body,data:{
      ...(isChapter ? {chapter:entry.id} : {}),
      ...(isOwnership ? {ownershipSteps} : {}),
      ...(quizzes[entry.slug] ? {answer:quizzes[entry.slug].answer} : {}),
    }});
    if (nativeLayout) spec.reader = {label, stageTitle:stage?.title ?? '', labelIcon:icon(stage?.icon ?? 'book',17),
      intro:firstPart, rest, outline, extra, outlineLinks,
      pagination: isChapter ? `<nav class="chapter-pagination" aria-label="相鄰章節">
        ${previous ? `<a href="${url(previous.path)}"><small>上一章</small><strong>${previous.title}</strong></a>` : `<a href="${url('guide')}"><small>開始之前</small><strong>學習指南</strong></a>`}
        ${next ? `<a href="${url(next.path)}"><small>下一章</small><strong>${next.title}</strong></a>` : `<a href="${url('resources')}"><small>繼續學習</small><strong>延伸資源</strong></a>`}</nav>` : '',
    };
    return spec;
  }

  function missing() {
    return layout({title:'找不到頁面',description:'頁面可能已移動。',current:'404',body:`<div class="home-page secondary-page"><h1>找不到頁面</h1><p>請從課程總覽選擇章節，或使用搜尋查找教材。</p>${cta('', '返回課程總覽')}</div>`});
  }

  return {home, lesson, resourceIndex, missing, url};
}
