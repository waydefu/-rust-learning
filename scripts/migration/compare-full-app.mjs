// Whole application comparison: actual old and native CLI commands, no generated
// HTML, search data or expected result is supplied to the Rust implementation.
import { execFileSync, spawnSync } from 'node:child_process';
import { readdir, readFile, mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import { projectRoot } from '../../src/course.mjs';
const root=join(projectRoot,'dist');
const base=(process.env.BASE_PATH||'/').replace(/\/?$/,'/');
const cargo=process.env.CARGO||'cargo';
const cli=['run','--quiet','--locked','--manifest-path','tools/site-render/Cargo.toml','--'];
const opts={cwd:projectRoot,encoding:'utf8',stdio:['ignore','pipe','inherit'],maxBuffer:64*1024*1024};
const old=(script)=>execFileSync(process.execPath,[script],opts);
const native=(command)=>execFileSync(cargo,[...cli,command],opts);
async function tree(dir=''){
  const result=new Map();
  const children=(await readdir(join(root,dir),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name));
  for(const child of children){
    const path=dir?dir+'/'+child.name:child.name;
    if(child.isDirectory())for(const [name,bytes] of await tree(path))result.set(name,bytes);
    else if(child.isFile())result.set(path,await readFile(join(root,path)));
    else throw new Error('意外的網站產物：'+path);
  }
  return result;
}
const summary=s=>s.split('\n').find(line=>line.includes('份 Markdown、')&&line.includes('站內引用'));
console.log(old('scripts/build.mjs').trim());
const oldCheck=old('scripts/check-legacy.mjs');
const expected=await tree();
console.log(native('build').trim());
const actual=await tree();
if(expected.size!==actual.size)throw new Error('完整 CLI 產物數量不一致');
for(const [path,bytes] of expected){
  const other=actual.get(path);
  if(!other)throw new Error('Rust CLI 缺少產物 '+path);
  // JSON object member order is not a browser contract. All remaining files,
  // including search-index.json and each HTML page, must be byte-identical.
  if(path==='build-render-report.json'){
    if(!isDeepStrictEqual(JSON.parse(bytes),JSON.parse(other)))throw new Error('完整 CLI 渲染報告不一致');
  }else if(!bytes.equals(other)){
    const n=bytes.findIndex((b,i)=>b!==other[i]);
    const at=n<0?Math.min(bytes.length,other.length):n;
    throw new Error('完整 CLI 產物不同 '+path+' at byte '+at+'\n原版 '+bytes.subarray(Math.max(0,at-80),at+180).toString()+'\nRust '+other.subarray(Math.max(0,at-80),at+180).toString());
  }
}
const nativeCheck=native('check');
if(!summary(oldCheck)||summary(nativeCheck)!==summary(oldCheck))throw new Error('完整 CLI 網站驗證結果不一致');
console.log(nativeCheck.trim());
const before=actual.get('index.html');
for(const invalidBase of ['//external/','/../','/invalid path/']){
  for(const [program,args] of [[process.execPath,['scripts/build.mjs']],[cargo,[...cli,'build']]]){
    const result=spawnSync(program,args,{cwd:projectRoot,env:{...process.env,BASE_PATH:invalidBase},encoding:'utf8',maxBuffer:64*1024*1024});
    if(result.error)throw result.error;
    if(!result.status)throw new Error('CLI 接受無效 BASE_PATH '+invalidBase);
    if(!(await readFile(join(root,'index.html'))).equals(before))throw new Error('失敗建置改動既有產物');
  }
}
const evidence=join(projectRoot,'.migration-evidence');
await mkdir(evidence,{recursive:true});
const hashes=[...actual].map(([path,bytes])=>({path,sha256:createHash('sha256').update(bytes).digest('hex')}));
await writeFile(join(evidence,base==='/'?'full-app-root.json':'full-app-pages.json'),JSON.stringify({base,files:actual.size,byteIdentical:actual.size-1,semanticJson:1,summary:summary(nativeCheck),invalidBases:3,fallback:0,hashes},null,2)+'\n');
console.log('完整 Rust CLI 與原版：'+(actual.size-1)+' 個產物逐位元組一致，渲染報告 JSON 一致，3/3 無效基礎路徑拒絕且保留舊產物');
