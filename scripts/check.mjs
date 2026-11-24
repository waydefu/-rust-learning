import { readdir, readFile } from 'node:fs/promises';
import { join, sep } from 'node:path';
import { parse } from 'parse5';
import { isDeepStrictEqual } from 'node:util';
import { catalog, chapters, projectRoot, readLesson, resources, stages } from '../src/course.mjs';
import { applyNativeFences, verifyNativeSite } from '../src/native-render.mjs';

const dist=join(projectRoot,'dist');
const base=(process.env.BASE_PATH||'/').replace(/\/?$/,'/');
const origin='https://rust-learning.test';
function* elements(node){
  if(node.tagName) yield node;
  for(const child of node.childNodes??[]) yield* elements(child);
}
const attrs=node=>Object.fromEntries(node.attrs.map(({name,value})=>[name,value]));
const textOf=node=>node?.nodeName==='#text'?node.value:(node?.childNodes??[]).map(textOf).join('');
const fields=[base,String(catalog.length)];
const lessons=await Promise.all(catalog.map(readLesson));
applyNativeFences(lessons);
const expectedIndex=[];
for(const lesson of lessons){
  const {entry}=lesson;
  fields.push(entry.id,entry.slug,entry.path,String(chapters.includes(entry)),String(resources.includes(entry)),
    String(stages.filter(s=>s.chapters.includes(entry)).length),entry.source.id,
    String(!Number.isNaN(Date.parse(entry.source.modified))&&Number.isInteger(entry.source.paragraphs)),
    String(lesson.headings.length));
  for(const h of lesson.headings) fields.push(String(h.level),h.label);
  const lines=lesson.source.split(/\r?\n/);
  fields.push(String(lesson.nativeFences.length));
  for(const token of lesson.nativeFences) fields.push(String(token.map[0]+1),token.markup,lines[token.map[1]-1]??'');
  const notes=lesson.tokens.map(t=>t.attrGet('data-callout')).filter(Boolean);
  fields.push(String(notes.length),...notes);
  expectedIndex.push({title:entry.title,id:entry.id,description:entry.description,href:base+entry.path+'/',
    headings:lesson.headings.filter(h=>h.level<=3).map(({id,label})=>({id,label})),text:lesson.fullText});
}
const names=(await readdir(dist,{recursive:true})).filter(name=>name.endsWith('.html'));
for(const name of names){
  const path=name.split(sep).join('/');
  const route=base+(path==='index.html'?'':path.replace(/\/index\.html$/,'/'));
  const nodes=[...elements(parse(await readFile(join(dist,name),'utf8')))];
  const ids=nodes.map(n=>attrs(n).id).filter(Boolean);
  fields.push(route,String(nodes.filter(n=>n.tagName==='h1').length),String(nodes.filter(n=>n.tagName==='main').length),
    attrs(nodes.find(n=>n.tagName==='html')).lang??'',String(nodes.filter(n=>n.tagName==='img'&&attrs(n).alt===undefined).length),
    String(ids.length),...ids);
  const notes=nodes.filter(n=>attrs(n)['data-callout']);
  fields.push(String(notes.length));
  for(const note of notes){
    const title=[...elements(note)].find(n=>attrs(n).class==='callout-title');
    fields.push(attrs(note)['data-callout'],attrs(note).role??'',textOf(title).trim());
  }
  const links=[];
  for(const node of nodes){
    const {href,src}=attrs(node);
    const address=href??src;
    if(!address)continue;
    const target=new URL(address,origin+route);
    if(target.origin!==origin)continue;
    links.push([address,target.pathname,target.hash?decodeURIComponent(target.hash.slice(1)):'']);
  }
  fields.push(String(links.length),...links.flat());
}
const report=JSON.parse(await readFile(join(dist,'build-render-report.json'),'utf8'));
fields.push(report.layout,String(report.pages),String(report.markdown.length));
for(const item of report.markdown) fields.push(item.slug,item.renderer);
const index=JSON.parse(await readFile(join(dist,'assets','search-index.json'),'utf8'));
if(!isDeepStrictEqual(index,expectedIndex))throw new Error('Rust 搜尋索引與原有資料契約不一致');
fields.push(String(index.length));
for(const entry of index)fields.push(entry.href,entry.text,String(entry.headings.length),...entry.headings.map(h=>h.id));
const [count,pages,links,bytes]=verifyNativeSite(names.length,fields);
console.log(count+' 份 Markdown、'+pages+' 個 HTML、'+links+' 個站內引用與搜尋索引通過，首頁與共用資源 '+(bytes/1000).toFixed(1)+' kB（未壓縮），BASE_PATH='+base);
