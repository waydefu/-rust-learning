import { execFileSync } from 'node:child_process';
import { projectRoot } from '../../src/course.mjs';
const run=path=>execFileSync(process.execPath,[path],{cwd:projectRoot,encoding:'utf8',stdio:['ignore','pipe','inherit'],maxBuffer:64*1024*1024});
const native=run('scripts/check-rust.mjs');
const legacy=run('scripts/check-rust-legacy.mjs');
const summary=s=>s.split('\n').filter(line=>line.startsWith('Rust 區塊通過 ')||line.startsWith('Cargo 專案通過 ')).sort();
if(JSON.stringify(summary(native))!==JSON.stringify(summary(legacy))||!summary(native).some(line=>line.startsWith('Rust 區塊通過 '))) throw new Error('Rust 範例執行統計與原版不一致');
console.log(native.trim());
console.log('Rust 範例執行器與原版統計、專案結果一致');
