import { execFileSync } from 'node:child_process';
import { parseFragment } from 'parse5';
import { md, projectRoot, renderTokens, chapters, resources, stages, highlightCode } from './course.mjs';
import { icon } from './icons.mjs';
import { flows, quizzes, ownershipSteps } from './learning-labs.mjs';
import { isDeepStrictEqual } from 'node:util';

function invoke(mode, count, fields) {
  const input = [mode, String(count), ...fields];
  if (input.some(value => typeof value !== 'string' || value.includes('\0'))) {
    throw new Error('Rust 渲染介面不接受非文字或 NUL 欄位');
  }
  const output = execFileSync(process.env.CARGO || 'cargo', [
    'run', '--quiet', '--locked', '--manifest-path', 'tools/site-render/Cargo.toml',
  ], {
    cwd: projectRoot, input: input.join('\0') + '\0', encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'inherit'], maxBuffer: 64 * 1024 * 1024,
  });
  const result = output.split('\0');
  if (result.pop() !== '' || result.shift() !== mode || result.shift() !== String(count)) {
    throw new Error('Rust 渲染回傳格式不相容');
  }
  return result;
}

// Compare parsed DOM, not formatting whitespace between block-level tags.
// Text inside pre/code is exact. Inline whitespace and every tag/attribute
// are preserved (apart from normalization of HTML formatting whitespace).
const blocks = new Set(['address','article','aside','blockquote','div','dl','dt','dd',
  'figcaption','figure','footer','h1','h2','h3','h4','h5','h6','header','hr',
  'li','main','nav','ol','p','pre','section','table','thead','tbody','tfoot','tr','td','th','ul']);
function normalized(node, exact = false) {
  if (node.nodeName === '#text') return node.value;
  if (node.nodeName === '#comment') return ['#comment', node.data];
  const tag = node.tagName || node.nodeName;
  const attrs = (node.attrs || []).map(({name,value}) => [name,
    name === 'style' ? value.replace(/\s+/g,'').replace(/;$/,'') : value,
  ]).sort((a,b) => a[0].localeCompare(b[0]));
  const preserve = exact || tag === 'pre' || tag === 'code';
  const children = [];
  for (const child of node.childNodes || []) {
    const item = normalized(child, preserve);
    if (typeof item === 'string' && typeof children.at(-1) === 'string') children[children.length - 1] += item;
    else children.push(item);
  }
  const isBlock = child => Array.isArray(child) && blocks.has(child[0]);
  if (!preserve) {
    for (let i = 0; i < children.length; i++) {
      if (typeof children[i] !== 'string') continue;
      let value = children[i].replace(/\s+/gu, ' ');
      if (i === 0 || isBlock(children[i - 1])) value = value.trimStart();
      if (i === children.length - 1 || isBlock(children[i + 1])) value = value.trimEnd();
      children[i] = value;
    }
  }
  return [tag, attrs, children.filter(child => child !== '')];
}

export const equivalentMarkup = (a, b) =>
  isDeepStrictEqual(normalized(parseFragment(a)), normalized(parseFragment(b)));

export function renderNativeBodies(lessons) {
  const fields = [];
  const codeReport = applyNativeHighlighting(lessons);
  let fenceIndex = 0;
  for (const lesson of lessons) {
    fields.push(lesson.source, String(lesson.headings.length));
    for (const h of lesson.headings) fields.push(String(h.level), h.id);
    const fences = lesson.tokens.filter(token => token.type === 'fence');
    fields.push(String(fences.length));
    for (const token of fences) {
      fields.push(token.info.trim(), token.content, codeReport.blocks[fenceIndex++]);
    }
  }
  const result = invoke('markdown-v1', lessons.length, fields);
  if (result.length !== lessons.length * 3) throw new Error('Rust 正文回傳數量不正確');
  const report = [];
  for (let i = 0; i < lessons.length; i++) {
    const lesson = lessons[i];
    const [status, intro, rest] = result.slice(i * 3, i * 3 + 3);
    const legacy = renderTokens(lesson.tokens);
    const compatible = status === 'ok' && equivalentMarkup(legacy, intro + rest);
    if (compatible) lesson.nativeBody = {intro, rest};
    report.push({
      slug: lesson.entry.slug,
      renderer: compatible ? 'rust' : 'legacy',
      reason: compatible ? '' : status === 'error' ? intro : 'DOM 與既有渲染器不同',
    });
  }
  const fallback = report.filter(item => item.renderer !== 'rust');
  console.log('Rust Markdown 渲染相容：' + (report.length - fallback.length) + '/' + report.length);
  for (const {slug, reason} of fallback) console.warn('保留既有渲染：' + slug + ' — ' + reason);
  if (fallback.length === report.length || (fallback.length && process.env.REQUIRE_NATIVE_MARKDOWN === '1')) {
    throw new Error('Rust Markdown 尚未達到指定相容性要求');
  }
  return report;
}

export function renderNativeLayouts(base, specs) {
  const fields = [
    base, icon('menu'), icon('search',19), icon('sun'), icon('moon'),
    icon('github'), icon('close'), icon('search'),
  ];
  for (const spec of specs) fields.push(spec.title, spec.description, spec.navigation, spec.body, spec.pageData);
  const result = invoke('layout-v1', specs.length, fields);
  if (result.length !== specs.length) throw new Error('Rust 頁面模板回傳數量不正確');
  return result;
}


// Rust extracts headings directly from source, then computes public anchors.
// The legacy parser's token offsets are retained for the old renderer.
export function applyNativeMetadata(lessons) {
  const result = invoke('metadata-v1', lessons.length, lessons.map(lesson => lesson.source));
  let cursor = 0;
  let total = 0;
  for (const lesson of lessons) {
    const title = result[cursor++];
    const count = Number(result[cursor++]);
    if (title !== lesson.title || count !== lesson.headings.length) {
      throw new Error('Rust 章節標題數量或名稱不一致：' + lesson.entry.slug);
    }
    const headings = [];
    for (const old of lesson.headings) {
      const level = Number(result[cursor++]);
      const label = result[cursor++];
      const id = result[cursor++];
      if (level !== old.level || label !== old.label || id !== old.id) {
        throw new Error('Rust 錨點與原版不一致：' + lesson.entry.slug + ' / ' + old.label);
      }
      headings.push({level,label,id,tokenIndex:old.tokenIndex});
      total++;
    }
    lesson.title = title;
    lesson.headings = headings;
  }
  if (cursor !== result.length) throw new Error('Rust 章節元資料含多餘欄位');
  console.log('Rust 章節元資料：' + lessons.length + ' 篇、' + total + ' 個錨點逐項比對通過');
}

export function nativeAnchors(groups) {
  const result = invoke('anchors-v1', groups.length, groups.flatMap(labels => [String(labels.length), ...labels]));
  let cursor = 0;
  const output = groups.map(labels => {
    if (Number(result[cursor++]) !== labels.length) throw new Error('Rust 錨點數量不一致');
    const ids = result.slice(cursor, cursor + labels.length);
    cursor += labels.length;
    return ids;
  });
  if (cursor !== result.length) throw new Error('Rust 錨點資料不一致');
  return output;
}

export function applyNativeNavigation(base, specs) {
  const fields = [base, String(resources.length), icon('home',19), icon('route',19),
    icon('layers',19), icon('down',16), icon('check',14), String(stages.length)];
  for (const stage of stages) {
    fields.push(stage.id,stage.title,String(stage.chapters.length));
    for (const entry of stage.chapters) fields.push(entry.id,entry.path,entry.title);
  }
  for (const spec of specs) fields.push(spec.current,spec.id);
  const result = invoke('navigation-v1',specs.length,fields);
  if (result.length !== specs.length) throw new Error('Rust 導覽數量不一致');
  for (let i=0;i<specs.length;i++) {
    if (result[i] !== specs[i].navigation) throw new Error('Rust 導覽與原版不一致：' + specs[i].current);
    specs[i].navigation = result[i];
  }
  console.log('Rust 導覽逐字比對：' + specs.length + ' 頁通過');
}

export function applyNativeLanding(base, specs) {
  const fields = [base, icon('chevron',14), icon('route',18), icon('book',16),
    icon('layers',16), icon('target',24), icon('list',17), icon('check',15), icon('chevron',15),
    chapters[0].path, String(chapters.length), String(resources.length)];
  for (const entry of resources) fields.push(entry.id, entry.path, entry.title, entry.description);
  fields.push(String(stages.length));
  for (const stage of stages) {
    fields.push(stage.id, stage.title, stage.description, icon(stage.icon,22), String(stage.chapters.length));
    for (const entry of stage.chapters) {
      fields.push(entry.id, entry.path, entry.title, entry.description, String(entry.slug === 'ownership'));
    }
  }
  const result = invoke('landing-v1', 2, fields);
  if (result.length !== 2 || specs[0].current !== '' || specs[1].current !== 'resources') {
    throw new Error('Rust 首頁與延伸資源回傳格式不一致');
  }
  for (let i=0; i<2; i++) {
    if (result[i] !== specs[i].body) throw new Error('Rust 首頁模板與原版不一致：' + specs[i].current);
    specs[i].body = result[i];
  }
  console.log('Rust 首頁與延伸資源模板逐字比對：2 頁通過');
}

export function applyNativeReaders(base, specs, lessons) {
  const fields = [base, icon('chevron',14),icon('list',17),icon('check',17),icon('target',18),icon('layers',17),icon('chevron',16)];
  for (let i=0; i<lessons.length; i++) {
    const {entry,title}=lessons[i], p=specs[i].reader;
    fields.push(entry.slug,String(chapters.includes(entry)),String(resources.includes(entry)),entry.title,title,entry.description,p.label,p.stageTitle,p.labelIcon,entry.source.id,p.intro,p.rest,p.outline,p.extra,p.outlineLinks,p.pagination);
    const ownership=entry.slug==='ownership';
    fields.push(String(ownership));
    if (ownership) for (const step of ownershipSteps) fields.push(step.code,step.message);
    const f=flows[entry.slug]; fields.push(String(Boolean(f)));
    if (f) fields.push(f.title,f.caption,String(f.nodes.length),...f.nodes);
    const q=quizzes[entry.slug]; fields.push(String(Boolean(q)));
    if(q) fields.push(q.prompt,q.explanation,String(q.choices.length),String(q.answer),...q.choices);
  }
  const result=invoke('reader-v1',lessons.length,fields);
  if(result.length!==specs.length) throw new Error('Rust 閱讀頁數量不一致');
  for(let i=0;i<specs.length;i++){
    if(result[i]!==specs[i].body) throw new Error('Rust 閱讀頁與原版不一致：'+specs[i].current);
    specs[i].body=result[i];
  }
  console.log('Rust 閱讀頁及互動模板逐字比對：'+specs.length+' 頁通過');
}

export function applyNativeHighlighting(lessons) {
  const tokens = lessons.flatMap(lesson => lesson.tokens.filter(token => token.type === 'fence'));
  const fields = [icon('copy',16)];
  for (const token of tokens) fields.push(token.info.trim(),token.content,highlightCode(token));
  const output = invoke('highlight-v1',tokens.length,fields);
  if(output.length!==tokens.length*2) throw new Error('Rust 程式區塊回傳數量不一致');
  let native=0;
  const blocks=tokens.map((token,i)=>{
    const original=md.renderer.rules.fence([token],0);
    const [candidate,wrapped]=output.slice(i*2,i*2+2);
    if(wrapped!==original) throw new Error('Rust 程式區塊外框不相容：'+token.info);
    if(candidate===original){native++;return candidate;}
    return wrapped;
  });
  console.log('Rust 高亮逐字相容：'+native+'/'+tokens.length+'；其餘保留原高亮，全部使用 Rust 程式區塊外框');
  return {blocks,native,total:tokens.length};
}

export function applyNativeFences(lessons) {
  const original=lessons.map(lesson=>lesson.tokens.filter(token=>token.type==='fence'));
  const fields=lessons.flatMap((lesson,i)=>[lesson.source,String(original[i].length)]);
  const result=invoke('fences-v1',lessons.length,fields);
  let cursor=0,total=0;
  for(let i=0;i<lessons.length;i++){
    const count=Number(result[cursor++]);
    if(count!==original[i].length) throw new Error('Rust 範例數量不同：'+lessons[i].entry.slug);
    lessons[i].nativeFences=original[i].map(token=>{
      const info=result[cursor++],content=result[cursor++];
      if(info!==token.info.trim()||content!==token.content) throw new Error('Rust 範例內容不同：'+lessons[i].entry.slug);
      total++;
      return {...token,info,content};
    });
  }
  if(cursor!==result.length) throw new Error('Rust 範例回傳多餘欄位');
  console.log('Rust 範例抽取與標注驗證：'+total+' 段逐字比對通過');
}

export function verifyNativeExamples(lessons) {
  const fields=[];
  for(const lesson of lessons){
    fields.push(lesson.entry.slug,String(lesson.nativeFences.length));
    for(const token of lesson.nativeFences) fields.push(String(token.map[0]+1),token.info,token.content);
  }
  const result=invoke('examples-v1',lessons.length,fields);
  const [version,compiled,run,error,project,messageCount,...messages]=result;
  if(![compiled,run,error,project,messageCount].every(x=>/^\d+$/.test(x))||messages.length!==Number(messageCount)){
    throw new Error('Rust 範例執行器回傳格式不一致');
  }
  return {version,compiled:Number(compiled),run:Number(run),error:Number(error),project:Number(project),messages};
}
