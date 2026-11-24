import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { dirname, join } from 'node:path';
import { catalog, chapters, md, projectRoot, readLesson } from '../src/course.mjs';
import { makeSite } from '../src/site.mjs';

const inputBase = process.env.BASE_PATH || '/';
if (!/^\/(?:[\w.-]+\/)*[\w.-]*\/?$/.test(inputBase) || inputBase.startsWith('//') || inputBase.split('/').some(segment => ['.', '..'].includes(segment))) {
  throw new Error('BASE_PATH 必須是 / 或以 / 開頭的安全部署子路徑');
}
const base = inputBase.endsWith('/') ? inputBase : inputBase + '/';
const dist = join(projectRoot, 'dist');
const site = makeSite(base);
// Native Rust preflight: reject malformed teaching notes before changing dist/.
execFileSync(process.env.CARGO || 'cargo', [
  'run', '--quiet', '--locked', '--manifest-path', 'tools/lesson-check/Cargo.toml', '--',
  ...chapters.map(entry => 'content/' + entry.slug + '.md'),
], { cwd: projectRoot, stdio: 'inherit' });
const lessons = await Promise.all(catalog.map(readLesson));

async function write(path, contents) {
  const target = join(dist, path);
  await mkdir(dirname(target), { recursive: true });
  await writeFile(target, contents);
}

await rm(dist, { recursive: true, force: true });
await mkdir(dist, { recursive: true });
await cp(join(projectRoot, 'src/assets'), join(dist, 'assets'), { recursive: true });
await Promise.all([
  write('index.html', site.home()),
  write('resources/index.html', site.resourceIndex()),
  write('404.html', site.missing()),
  write('.nojekyll', ''),
  ...lessons.map(lesson => write(`${lesson.entry.path}/index.html`, site.lesson(lesson))),
  write('assets/search-index.json', JSON.stringify(lessons.map(({entry, headings, fullText}) => ({
    title:entry.title, id:entry.id, description:entry.description, href:site.url(entry.path),
    headings:headings.filter(heading => heading.level <= 3).map(({id,label}) => ({id,label})), text:fullText,
  })))),
  write('third-party-licenses.txt', (await Promise.all(['markdown-it','highlight.js'].map(async name =>
    `${name}\n${await readFile(join(projectRoot,'node_modules',name,'LICENSE'), 'utf8')}`))).join('\n\n')),
]);

console.log(`Built ${lessons.length + 3} HTML pages at ${base} (MarkdownIt, CommonMark)`);
