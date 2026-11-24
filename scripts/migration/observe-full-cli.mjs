// rust-anything whole CLI observation adapter, not a production dependency.
// Run the real program in a fresh isolated course tree; expose stable status,
// output file hashes, validation summaries and HTTP responses to the recorder.
import { cp, mkdir, mkdtemp, readFile, readdir, rm, symlink, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createServer } from 'node:net';
import { once } from 'node:events';
const [implementation,operation,base]=process.argv.slice(2);
const root=process.env.RUST_LEARNING_REPLAY_ROOT;
const target=process.env.RUST_LEARNING_REPLAY_TARGET;
if(!root||!target||!['legacy','native'].includes(implementation)||!base)throw new Error('Invalid replay adapter input');
const work=await mkdtemp(join(tmpdir(),'rust-course-app-'));
const env={...process.env,BASE_PATH:base,CARGO_TARGET_DIR:target,VERIFY_SITE_WRITE:'0'};
const cli=['run','--quiet','--locked','--manifest-path','tools/site-render/Cargo.toml','--'];
const commands={build:'scripts/build.mjs',check:'scripts/check-legacy.mjs',examples:'scripts/check-rust-legacy.mjs',dev:'scripts/dev.mjs'};
const args=command=>implementation==='legacy'?[process.execPath,[commands[command]]]:[process.env.CARGO||'cargo',[...cli,command==='examples'?'check-rust':command]];
function run(command,expectSuccess=true){
  const [program,argv]=args(command);
  const result=spawnSync(program,argv,{cwd:work,env,encoding:'utf8',maxBuffer:64*1024*1024,timeout:180000});
  if(result.error)throw result.error;
  if(result.status===null||(expectSuccess&&result.status!==0))throw new Error(command+' failed: '+result.stderr+'\n'+result.stdout);
  return result;
}
const canonical=value=>Array.isArray(value)?value.map(canonical):value!==null&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])])):value;
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
async function files(dir=''){
  const result=[];
  for(const item of (await readdir(join(work,'dist',dir),{withFileTypes:true})).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0)){
    const path=dir?dir+'/'+item.name:item.name;
    if(item.isDirectory())result.push(...await files(path));
    else if(item.isFile()){
      let bytes=await readFile(join(work,'dist',path));
      if(path==='build-render-report.json')bytes=Buffer.from(JSON.stringify(canonical(JSON.parse(bytes))));
      result.push({path,sha256:hash(bytes)});
    }else throw new Error('Unexpected output file '+path);
  }
  return result;
}
async function availablePort(){
  const server=createServer();server.listen(0,'127.0.0.1');await once(server,'listening');
  const port=server.address().port;await new Promise((resolve,reject)=>server.close(error=>error?reject(error):resolve()));return port;
}
async function preview(){
  const port=await availablePort();const [program,argv]=args('dev');
  const proc=spawn(program,argv,{cwd:work,env:{...env,HOST:'127.0.0.1',PORT:String(port)},stdio:['ignore','pipe','pipe'],detached:true});
  let log='',error;
  proc.stdout.on('data',b=>log+=b);proc.stderr.on('data',b=>log+=b);proc.on('error',e=>error=e);
  const done=once(proc,'close').catch(()=>{});
  try{
    const url='http://127.0.0.1:'+port;
    let ready=false;
    for(let i=0;i<300;i++){
      if(error||proc.exitCode!==null)throw error||new Error('Preview exited: '+log);
      try{const res=await fetch(url+base,{signal:AbortSignal.timeout(1000)});if(res.ok){await res.arrayBuffer();ready=true;break}}catch{}
      await new Promise(resolve=>setTimeout(resolve,200));
    }
    if(!ready)throw new Error('Preview not ready: '+log);
    const cases=[['GET',''],['GET','chapters/ownership/'],['GET','chapters/ownership'],['GET','not-here/'],['HEAD','assets/site.css'],['GET','assets/search-index.json']];
    const responses=[];
    for(const [method,path] of cases){
      const res=await fetch(url+base+path,{method,redirect:'manual',signal:AbortSignal.timeout(10000)});
      responses.push({method,path,status:res.status,type:res.headers.get('content-type'),location:res.headers.get('location'),sha256:hash(Buffer.from(await res.arrayBuffer()))});
    }
    return responses;
  }finally{
    if(proc.pid)try{process.kill(-proc.pid,'SIGTERM')}catch{}
    await done;
  }
}
try{
  for(const name of ['content','src','scripts','examples','tools']){
    await cp(join(root,name),join(work,name),{recursive:true,filter:path=>!path.split('/').includes('target')});
  }
  await cp(join(root,'rust-toolchain.toml'),join(work,'rust-toolchain.toml'));
  if(implementation==='legacy')await symlink(join(root,'node_modules'),join(work,'node_modules'),'dir');
  let observation;
  if(operation==='build'){
    run('build');const result=run('check');
    const summary=result.stdout.split('\n').find(line=>line.includes('份 Markdown、')&&line.includes('站內引用'));
    if(!summary)throw new Error('Missing validation summary');
    observation={operation,base,summary,files:await files()};
  }else if(operation==='invalid'||operation==='missing-source'){
    await mkdir(join(work,'dist'));await writeFile(join(work,'dist','index.html'),'previous 🦀');
    if(operation==='missing-source')await rm(join(work,'content','ownership.md'));
    const result=run('build',false);
    if(result.status===0)throw new Error('Invalid build unexpectedly succeeded');
    observation={operation,base,status:result.status,preserved:(await readFile(join(work,'dist','index.html'),'utf8'))==='previous 🦀',files:await files()};
  }else if(operation==='examples'){
    const result=run('examples');
    const lines=result.stdout.split('\n').filter(line=>line.startsWith('Rust 區塊通過 ')||line.startsWith('Cargo 專案通過 ')).sort();
    if(!lines.some(line=>line.startsWith('Rust 區塊通過 ')))throw new Error('Missing example summary');
    observation={operation,lines};
  }else if(operation==='preview'){
    observation={operation,base,responses:await preview()};
  }else throw new Error('Unknown replay operation');
  await writeFile('observation.json',JSON.stringify(observation,null,2)+'\n');
  console.log('Observed '+operation+' '+base);
}finally{await rm(work,{recursive:true,force:true});}
