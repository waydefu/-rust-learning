import { spawnSync } from 'node:child_process';
import { closeSync, mkdirSync, mkdtempSync, openSync, readFileSync, rmSync, writeFileSync, copyFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { tmpdir } from 'node:os';
import { catalog, projectRoot, readLesson } from '../src/course.mjs';

import { applyNativeFences } from '../src/native-render.mjs';

const root = mkdtempSync(join(tmpdir(), 'rust-lessons-'));
const rustc = process.env.RUSTC || 'rustc';
const cargo = process.env.CARGO || 'cargo';
const stats = { compiled: 0, run: 0, error: 0, project: 0 };
const projects = new Map();
const failures = [];
const fixture = join(root, 'snippet.rs');
const binary = join(root, process.platform === 'win32' ? 'snippet.exe' : 'snippet');
const log = join(root, 'command.log');

function command(program, args, cwd = root, timeout = 20_000) {
  const fd = openSync(log, 'w');
  try {
    const result = spawnSync(program, args, { cwd, timeout, stdio: ['ignore', fd, fd] });
    if (result.error) throw result.error;
    return {status:result.status, output:readFileSync(log,'utf8')};
  } finally { closeSync(fd); }
}

function assertSuccess(result) {
  if (result.status !== 0) throw new Error(result.output || `程式結束狀態 ${result.status}`);
}

function asBinary(source) {
  return /^\s*(?:async\s+)?fn\s+main\s*\(/m.test(source) ? source : `fn main() {\n${source}\n}\n`;
}

function collectProject(argument, source) {
  const [name, path] = argument?.split(':') ?? [];
  if (!/^[a-z][a-z_]*$/.test(name) || !/^src\/(?:bin\/)?[a-z_]+\.rs$/.test(path)) throw new Error('專案範例需指定 name:src/file.rs');
  if (!projects.has(name)) projects.set(name, new Map());
  const files = projects.get(name);
  if (files.has(path)) throw new Error(`重複的專案範例 ${name}:${path}`);
  files.set(path, name === 'external' ? asBinary(source) : source);
}

function verifySnippet(mode, argument, source, previous) {
  const flags = ['--edition=2024','-Awarnings'];
  if (mode === 'project') {
    collectProject(argument, source);
    return;
  }
  if (mode === 'continuation') {
    if (!previous) throw new Error('接續範例的前一個區塊必須是 Rust');
    source = previous + '\n' + source;
  }
  if (mode === 'program' && !/^fn\s+main\s*\(/m.test(source)) throw new Error('完整程式缺少 fn main()');
  writeFileSync(fixture, mode === 'test' ? source : asBinary(source));
  if (mode === 'error') {
    if (!/^E\d{4}(?:,E\d{4})*$/.test(argument)) throw new Error('需指定至少一個預期診斷碼');
    const result = command(rustc, [...flags, '--emit=metadata', '--error-format=json', '-o', binary, fixture]);
    const errors = result.output.trim().split('\n').flatMap(line => {
      if (!line.startsWith('{')) return [];
      const diagnosis = JSON.parse(line);
      return diagnosis.level === 'error' && diagnosis.code ? [diagnosis.code.code] : [];
    });
    const expected = argument.split(',');
    if (!result.status || expected.some(code => !errors.includes(code)) || errors.some(code => !expected.includes(code))) {
      throw new Error(`預期 ${argument}，實際 ${errors.join(', ') || '無診斷碼'}\n${result.output}`);
    }
    stats.error++;
    return;
  }
  if (!['run','test','fragment','continuation','program'].includes(mode)) throw new Error(`未知的 Rust 範例類型 ${mode}`);
  if (mode === 'test') flags.push('--test');
  if (mode === 'fragment' || mode === 'continuation' || mode === 'program') flags.push('--emit=metadata');
  assertSuccess(command(rustc, [...flags, '-o', binary, fixture]));
  stats.compiled++;
  if (mode === 'run' || mode === 'test') {
    assertSuccess(command(binary, []));
    stats.run++;
  }
}

function verifyProjects() {
  for (const [name, files] of projects) {
    const project = join(root,name);
    mkdirSync(project, {recursive:true});
    if (name === 'external') {
      for (const file of ['Cargo.toml','Cargo.lock']) copyFileSync(join(projectRoot,'examples',file), join(project,file));
    } else {
      writeFileSync(join(project,'Cargo.toml'), `[package]\nname = "${name}"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n`);
    }
    for (const [path, source] of files) {
      const target = join(project,path);
      mkdirSync(dirname(target), {recursive:true});
      writeFileSync(target,source);
    }
    try {
      if (name === 'external') {
        assertSuccess(command(cargo,['test','--locked','--all-targets','--quiet'],project,180_000));
        for (const bin of ['random','timer']) {
          assertSuccess(command(cargo,['run','--locked','--quiet','--bin',bin],project,180_000));
        }
      } else {
        assertSuccess(command(cargo,['test','--offline','--quiet'],project,60_000));
      }
      stats.project += files.size;
      console.log(`Cargo 專案通過 ${name}（${files.size} 段）`);
    } catch (error) { failures.push(`${name}: ${error.message}`); }
  }
}

try {
  // Rustup resolves the toolchain from the compiler and Cargo working directory.
  copyFileSync(join(projectRoot,'rust-toolchain.toml'), join(root,'rust-toolchain.toml'));
  const version = command(rustc,['--version']);
  assertSuccess(version);
  console.log(version.output.trim());
  const lessons = await Promise.all(catalog.map(readLesson));
  applyNativeFences(lessons);
  for (const lesson of lessons) {
    const {entry} = lesson;
    let previous;
    for (const token of lesson.nativeFences) {
      const [language, mode, argument] = token.info.trim().split(/\s+/);
      if (language !== 'rust') { previous = undefined; continue; }
      try { verifySnippet(mode, argument, token.content, previous); }
      catch (error) { failures.push(`${entry.slug}.md:${token.map[0]+1} ${error.message}`); }
      previous = token.content;
    }
  }
  verifyProjects();
  console.log(`Rust 區塊通過 ${stats.compiled + stats.error + stats.project} 段（獨立編譯 ${stats.compiled} 段，其中 ${stats.run} 段執行或執行測試，預期診斷 ${stats.error} 段，Cargo 專案 ${stats.project} 段）`);
  if (failures.length) {
    console.error(failures.join('\n\n'));
    process.exitCode = 1;
  }
} finally {
  rmSync(root, {recursive:true,force:true});
}
