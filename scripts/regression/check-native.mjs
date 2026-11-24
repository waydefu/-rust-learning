// Independent native regression tests; no Node renderer or historical oracle.
import { execFileSync, spawnSync } from 'node:child_process';
import { readdir, readFile, writeFile, mkdir, rename } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { gunzipSync } from 'node:zlib';
import { projectRoot } from '../../tests/support/course.mjs';

const cargo=process.env.CARGO || 'cargo';
const cli=['run','--quiet','--locked','--manifest-path','tools/site-render/Cargo.toml','--'];
const dist=join(projectRoot,'dist');
const opts={cwd:projectRoot,encoding:'utf8',maxBuffer:64*1024*1024};
const run=cmd=>execFileSync(cargo,[...cli,cmd],opts);
async function manifest(folder='') {
  const entries=(await readdir(join(dist,folder),{withFileTypes:true})).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0);
  const out=[];
  for (const item of entries) {
    const path=folder?folder+'/'+item.name:item.name;
    if(item.isDirectory()) out.push(...await manifest(path));
    else if(item.isFile()) out.push([path,createHash('sha256').update(await readFile(join(dist,path))).digest('hex')]);
    else throw new Error('Unexpected non-regular artifact '+path);
  }
  return out;
}
console.log(run('build').trim());
console.log(run('check').trim());
const base=process.env.BASE_PATH || '/';
const actual=(await manifest()).sort(([a],[b])=>a<b?-1:a>b?1:0);
const baseline=JSON.stringify(actual);
const historical=gunzipSync(await readFile(join(projectRoot,'tests/fixtures/site-baseline.json.gz')));
const reference=JSON.parse(historical)[base];
if (!reference || reference.base!==base) throw new Error('No approved output baseline for '+base);
const expected=new Map(reference.hashes.map(({path,sha256})=>[path,sha256]));
const current=new Map(actual);
const candidateDir=join(projectRoot,'.regression-evidence');
await mkdir(candidateDir,{recursive:true});
await writeFile(join(candidateDir,base==='/'?'root.json':'pages.json'),
  JSON.stringify({base,files:actual.length,hashes:actual.map(([path,sha256])=>({path,sha256}))},null,2)+'\n');
const differences=[...new Set([...expected.keys(),...current.keys()])].filter(p=>expected.get(p)!==current.get(p));
if(differences.length)throw new Error('Output differs from approved baseline:\n'+differences.map(p=>p+' expected='+expected.get(p)+' actual='+current.get(p)).join('\n'));
console.log('Approved output baseline: '+actual.length+'/'+reference.files+' files match exactly');
run('build');
if (JSON.stringify((await manifest()).sort(([a],[b])=>a<b?-1:a>b?1:0))!==baseline) throw new Error('Rust build is nondeterministic');
for(const invalid of ['//external/','/../','/invalid path/']){
  const result=spawnSync(cargo,[...cli,'build'],{...opts,env:{...process.env,BASE_PATH:invalid}});
  if(result.error)throw result.error;
  if(result.status===null||result.status===0)throw new Error('Accepted invalid BASE_PATH '+invalid);
  if(JSON.stringify((await manifest()).sort(([a],[b])=>a<b?-1:a>b?1:0))!==baseline)throw new Error('Failed build changed prior output');
}
if(base==='/'){
  // Reject missing content and invalid JSON schemas without changing dist.
  const lesson=join(projectRoot,'content/ownership.md');
  const moved=lesson+'.regression-missing';
  await rename(lesson,moved);
  try{
    const result=spawnSync(cargo,[...cli,'build'],opts);
    if(result.error)throw result.error;
    if(result.status===null||result.status===0)throw new Error('Build accepted missing content');
  }finally{await rename(moved,lesson);}
  const dataPath=join(projectRoot,'content/site-data.json');
  const originalData=await readFile(dataPath);
  for(const mutation of [
    data=>{data.quizzes['getting-started'].answer='0';},
    data=>{data.stages[0].typo=true;},
    data=>{data.ownershipSteps.pop();},
  ]){
    const data=JSON.parse(originalData);
    mutation(data);
    try{
      await writeFile(dataPath,JSON.stringify(data));
      const result=spawnSync(cargo,[...cli,'build'],opts);
      if(result.error)throw result.error;
      if(result.status===null||result.status===0)throw new Error('Build accepted invalid source schema');
    }finally{await writeFile(dataPath,originalData);}
  }
  if(JSON.stringify((await manifest()).sort(([a],[b])=>a<b?-1:a>b?1:0))!==baseline)throw new Error('Invalid source changed previous output');
  const cases=[
    ['index.html',s=>s.replace('<main id="main">','<main id="main"><p id="main">duplicate</p>'),'有重複 id'],
    ['index.html',s=>s.replace('href="#main"','href="#missing-anchor"'),'錨點不存在'],
    ['index.html',s=>s.replace('/assets/site.css','/assets/missing.css'),'資源或頁面不存在'],
    ['index.html',s=>s.replace('/assets/modules/context.js','/assets/modules/missing.js'),'資源或頁面不存在'],
    ['chapters/ownership/index.html',s=>s.replace('role="note"','role="presentation"'),'教學方塊缺少文字標題或語意'],
    ['assets/site.css',s=>s+'\n/*'+'x'.repeat(161000)+'*/','首頁與共用資源超過 160 kB'],
    ['assets/search-index.json',s=>{const v=JSON.parse(s);v[0].title+='（錯誤）';return JSON.stringify(v);},'Rust 搜尋索引與原有資料契約不一致'],
    ['build-render-report.json',s=>{const v=JSON.parse(s);v.layout='legacy';return JSON.stringify(v);},'Rust 渲染報告不完整'],
  ];
  for(const [name,mutate,expected] of cases){
    const path=join(dist,name);
    const original=await readFile(path);
    const changed=mutate(original.toString('utf8'));
    if(changed===original.toString('utf8'))throw new Error('Mutation did not change '+name);
    try{
      await writeFile(path,changed);
      const result=spawnSync(cargo,[...cli,'check'],opts);
      if(result.error)throw result.error;
      if(result.status===null||result.status===0||!(result.stdout+result.stderr).includes(expected)){
        throw new Error('Native checker accepted '+name+' / '+expected+'\n'+result.stdout+'\n'+result.stderr);
      }
    }finally{await writeFile(path,original);}
  }
  console.log('Native checker rejected '+cases.length+'/'+cases.length+' negative fixtures');
}
if(JSON.stringify((await manifest()).sort(([a],[b])=>a<b?-1:a>b?1:0))!==baseline)throw new Error('Regression fixtures were not restored');
console.log('Deterministic build, rejected invalid bases, unchanged full output tree');
