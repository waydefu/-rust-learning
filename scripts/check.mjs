import { readdir, readFile, stat } from 'node:fs/promises';
import { join, relative, sep } from 'node:path';
import { parse } from 'parse5';
import { catalog, chapters, projectRoot, readLesson, resources, stages } from '../src/course.mjs';

const dist = join(projectRoot, 'dist');
const base = (process.env.BASE_PATH || '/').replace(/\/?$/, '/');
const origin = 'https://rust-learning.test';
const errors = [];
const routes = new Map();
const unique = (values) => new Set(values).size === values.length;
const assets = new Set();
let checkedLinks = 0;

function* elements(node) {
  if (node.tagName) yield node;
  for (const child of node.childNodes ?? []) yield* elements(child);
}
const attrs = node => Object.fromEntries(node.attrs.map(({name,value})=>[name,value]));

if (chapters.length !== 21 || chapters.some((entry,index) => Number(entry.id) !== index + 1)) errors.push('章節必須依序涵蓋 01–21');
if (resources.length !== 8 || resources.some((entry,index) => Number(entry.id.slice(1)) !== index + 1)) errors.push('附錄必須依序涵蓋 A01–A08');
if (!['id','slug','path'].every(field => unique(catalog.map(entry=>entry[field])))) errors.push('課程識別碼、slug 與路徑不可重複');
if (stages.flatMap(stage=>stage.chapters).length !== chapters.length) errors.push('每章必須屬於一個階段');

const sourceFiles = (await readdir(join(projectRoot,'content'))).filter(name => name.endsWith('.md'));
if (sourceFiles.length !== catalog.length || catalog.some(entry=>!sourceFiles.includes(entry.slug + '.md'))) errors.push('Markdown 檔案與 catalog.json 不一致');

for (const entry of catalog) {
  if (!/^[\w-]{20,}$/.test(entry.source.id) || Number.isNaN(Date.parse(entry.source.modified)) || !Number.isInteger(entry.source.paragraphs)) errors.push(`${entry.slug} 缺少有效的來源記錄`);
  const lesson = await readLesson(entry);
  const lines = (await readFile(join(projectRoot,'content',entry.slug+'.md'),'utf8')).split(/\r?\n/);
  if (!lesson.headings.length) errors.push(`${entry.slug} 缺少內容標題`);
  let headingLevel = 1;
  for (const heading of lesson.headings) {
    if (heading.level > headingLevel + 1) errors.push(`${entry.slug} 標題層級跳躍 ${heading.label}`);
    headingLevel = heading.level;
  }
  for (const token of lesson.tokens.filter(token=>token.type==='fence')) {
    const start = token.map[0] + 1;
    const ending = lines[token.map[1] - 1].trim();
    if (!ending.startsWith(token.markup) || [...ending].some(char=>char!==token.markup[0])) errors.push(`${entry.slug}.md:${start} 缺少程式碼結尾`);
    if (!/^(?:rust (?:run|program|fragment|continuation|test|error E\d{4}(?:,E\d{4})*|project [a-z][a-z_]*:src\/(?:bin\/)?[a-z_]+\.rs)|text|bash|toml)$/.test(token.info)) errors.push(`${entry.slug}.md:${start} 無效的範例標注`);
  }
}

for (const name of (await readdir(dist, {recursive:true})).filter(name => name.endsWith('.html'))) {
  const path = name.split(sep).join('/');
  const route = base + (path === 'index.html' ? '' : path.replace(/\/index\.html$/,'/'));
  const html = await readFile(join(dist,name),'utf8');
  const nodes = [...elements(parse(html))];
  const ids = nodes.map(node=>attrs(node).id).filter(Boolean);
  if (!unique(ids)) errors.push(`${route} 有重複 id`);
  if (nodes.filter(node=>node.tagName==='h1').length!==1) errors.push(`${route} 必須包含單一 H1`);
  if (nodes.filter(node=>node.tagName==='main').length!==1 || !ids.includes('main')) errors.push(`${route} 缺少內容地標`);
  if (attrs(nodes.find(node=>node.tagName==='html')).lang !== 'zh-Hant') errors.push(`${route} 缺少 zh-Hant`);
  routes.set(route, {nodes, ids:new Set(ids)});
}
for (const route of [base, base+'resources/', base+'404.html', ...catalog.map(entry=>base+entry.path+'/')]) {
  if (!routes.has(route)) errors.push(`${route} 尚未生成`);
}

for (const [route, page] of routes) {
  for (const node of page.nodes) {
    const {href,src,alt} = attrs(node);
    if (node.tagName === 'img' && alt === undefined) errors.push(`${route} 圖片缺少替代文字`);
    const address = href ?? src;
    if (!address) continue;
    const target = new URL(address, origin + route);
    if (target.origin !== origin) continue;
    checkedLinks++;
    if (!target.pathname.startsWith(base)) {
      errors.push(`${route}: ${address} 超出部署子路徑`);
      continue;
    }
    if (routes.has(target.pathname)) {
      if (target.hash && !routes.get(target.pathname).ids.has(decodeURIComponent(target.hash.slice(1)))) errors.push(`${route}: 錨點不存在 ${address}`);
    } else {
      const name = target.pathname.slice(base.length);
      if (!(await stat(join(dist,name)).catch(()=>null))?.isFile()) errors.push(`${route}: 資源或頁面不存在 ${address}`);
      else assets.add(name);
    }
  }
}

const index = JSON.parse(await readFile(join(dist,'assets','search-index.json'),'utf8'));
if (index.length !== catalog.length || index.some(entry=>!routes.has(entry.href)||!entry.text||entry.headings.some(heading=>!routes.get(entry.href).ids.has(heading.id)))) {
  errors.push('搜尋索引的頁面或章節定位不完整');
}
const initial = ['index.html',...assets].filter(name => !name.endsWith('.json'));
const rawBytes = (await Promise.all(initial.map(name=>stat(join(dist,name))))).reduce((total, entry)=>total+entry.size,0);
if (rawBytes > 160_000) errors.push(`首頁與共用資源超過 160 kB（實際 ${rawBytes} bytes）`);

if (errors.length) {
  console.error(errors.join('\n'));
  process.exitCode = 1;
} else {
  console.log(`${catalog.length} 份 Markdown、${routes.size} 個 HTML、${checkedLinks} 個站內引用與搜尋索引通過，首頁與共用資源 ${(rawBytes/1000).toFixed(1)} kB（未壓縮），BASE_PATH=${base}`);
}
