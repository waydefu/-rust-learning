import { execFileSync, spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { projectRoot } from '../../src/course.mjs';
const run=path=>execFileSync(process.execPath,[path],{cwd:projectRoot,encoding:'utf8',stdio:['ignore','pipe','inherit'],maxBuffer:64*1024*1024});
const native=run('scripts/check.mjs');
const legacy=run('scripts/check-legacy.mjs');
const summary=s=>s.split('\n').find(line=>line.includes('份 Markdown、')&&line.includes('站內引用'));
if(!summary(native)||summary(native)!==summary(legacy))throw new Error('Rust 網站驗證統計與原版不一致');
console.log(native.trim());
console.log('Rust 網站驗證統計與原版一致');

if((process.env.BASE_PATH||'/')==='/'){
  const cases=[
    ['index.html',s=>s.replace('<main id="main">','<main id="main"><p id="main">duplicate</p>'),'有重複 id'],
    ['index.html',s=>s.replace('href="#main"','href="#migration-missing-anchor"'),'錨點不存在'],
    ['index.html',s=>s.replace('/assets/site.css','/assets/migration-missing.css'),'資源或頁面不存在'],
    ['chapters/ownership/index.html',s=>s.replace('role="note"','role="presentation"'),'教學方塊缺少文字標題或語意'],
    ['assets/site.css',s=>s+'\n/*'+ 'x'.repeat(161000)+'*/','首頁與共用資源超過 160 kB'],
    ['assets/search-index.json',s=>{const v=JSON.parse(s);v[0].title+='（錯誤）';return JSON.stringify(v);},'Rust 搜尋索引與原有資料契約不一致'],
    ['build-render-report.json',s=>{const v=JSON.parse(s);v.layout='legacy';return JSON.stringify(v);},'Rust 渲染報告不完整']
  ];
  for(const [name,mutate,expected] of cases){
    const path=join(projectRoot,'dist',name);
    const original=await readFile(path);
    const changed=mutate(original.toString('utf8'));
    if(changed===original.toString('utf8'))throw new Error('負例沒有修改任何內容：'+name);
    try{
      await writeFile(path,changed,'utf8');
      for(const script of ['scripts/check.mjs','scripts/check-legacy.mjs']){
        const result=spawnSync(process.execPath,[script],{cwd:projectRoot,encoding:'utf8',maxBuffer:64*1024*1024});
        if(result.error)throw result.error;
        if(result.status===null||result.status===0||!(result.stdout+result.stderr).includes(expected)){
          throw new Error(script+' 未拒絕負例 '+name+' / '+expected+'\n'+result.stdout+'\n'+result.stderr);
        }
      }
    }finally{await writeFile(path,original);}
  }
  console.log('Rust 與原版網站驗證拒絕 '+cases.length+'/'+cases.length+' 個負例，產物已還原');
}
