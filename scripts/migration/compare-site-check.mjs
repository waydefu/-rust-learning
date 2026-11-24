import { execFileSync } from 'node:child_process';
import { projectRoot } from '../../src/course.mjs';
const run=path=>execFileSync(process.execPath,[path],{cwd:projectRoot,encoding:'utf8',stdio:['ignore','pipe','inherit'],maxBuffer:64*1024*1024});
const native=run('scripts/check.mjs');
const legacy=run('scripts/check-legacy.mjs');
const summary=s=>s.split('\n').find(line=>line.includes('份 Markdown、')&&line.includes('站內引用'));
if(!summary(native)||summary(native)!==summary(legacy))throw new Error('Rust 網站驗證統計與原版不一致');
console.log(native.trim());
console.log('Rust 網站驗證統計與原版一致');
