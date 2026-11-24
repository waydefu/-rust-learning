import { readFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';
import { catalog, chapters, md, projectRoot, readLesson } from '../src/course.mjs';
import { makeSite } from '../src/site.mjs';
import { applyNativeMetadata, applyNativeNavigation, applyNativeLanding, applyNativeReaders, renderNativeBodies, renderNativeLayouts } from '../src/native-render.mjs';

const inputBase = process.env.BASE_PATH || '/';
if (!/^\/(?:[\w.-]+\/)*[\w.-]*\/?$/.test(inputBase) || inputBase.startsWith('//') || inputBase.split('/').some(segment => ['.', '..'].includes(segment))) {
  throw new Error('BASE_PATH 必須是 / 或以 / 開頭的安全部署子路徑');
}
const base = inputBase.endsWith('/') ? inputBase : inputBase + '/';
const site = makeSite(base);
// Native Rust preflight: reject malformed teaching notes before changing dist/.
execFileSync(process.env.CARGO || 'cargo', [
  'run', '--quiet', '--locked', '--manifest-path', 'tools/lesson-check/Cargo.toml', '--',
  ...chapters.map(entry => 'content/' + entry.slug + '.md'),
], { cwd: projectRoot, stdio: 'inherit' });
const lessons = await Promise.all(catalog.map(readLesson));
applyNativeMetadata(lessons);
const renderReport = renderNativeBodies(lessons);

// Keep Markdown parsing in the existing renderer for now. Native Rust owns
// the search index's JSON schema and heading selection.
const searchFields = ['rust-search-v1', String(lessons.length)];
for (const {entry, headings, fullText} of lessons) {
  searchFields.push(entry.title, entry.id, entry.description, site.url(entry.path),
    fullText, String(headings.length));
  for (const {id, label, level} of headings) searchFields.push(id, label, String(level));
}
if (searchFields.some(field => typeof field !== 'string' || field.includes('\0'))) {
  throw new Error('搜尋欄位含有不支援的 NUL 字元或非文字資料');
}
const searchIndex = execFileSync(process.env.CARGO || 'cargo', [
  'run', '--quiet', '--locked', '--manifest-path', 'tools/site-tools/Cargo.toml',
  '--bin', 'search-index',
], {
  cwd: projectRoot, input: searchFields.join('\0') + '\0', encoding: 'utf8',
  maxBuffer: 32 * 1024 * 1024, stdio: ['pipe', 'pipe', 'inherit'],
});

// Native Rust constructs the HTML document shell for every route. Keep the
// previous layout as an exact-output oracle until migration is established.
const nativeSite = makeSite(base, {nativeLayout: true});
const pagePaths = ['index.html', 'resources/index.html', '404.html', ...lessons.map(l => l.entry.path + '/index.html')];
const specs = [nativeSite.home(), nativeSite.resourceIndex(), nativeSite.missing(), ...lessons.map(nativeSite.lesson)];
const baseline = [site.home(), site.resourceIndex(), site.missing(), ...lessons.map(site.lesson)];
applyNativeNavigation(base, specs);
applyNativeLanding(base, specs);
applyNativeReaders(base, specs.slice(3), lessons);
const pages = renderNativeLayouts(base, specs);
for (let i = 0; i < pages.length; i++) {
  if (pages[i] !== baseline[i]) throw new Error('Rust 頁面模板與原版不一致：' + pagePaths[i]);
}

// Rust owns the filesystem output. The complete packet is validated before
// replacing dist, and the existing output survives failed staging.
const generated = [
  ...pagePaths.map((path,i) => [path, pages[i]]),
  ['.nojekyll', ''],
  ['assets/search-index.json', searchIndex],
  ['third-party-licenses.txt', (await Promise.all(['markdown-it','highlight.js'].map(async name =>
    name + '\n' + await readFile(join(projectRoot,'node_modules',name,'LICENSE'), 'utf8')))).join('\n\n')],
  ['build-render-report.json', JSON.stringify({layout: 'rust', pages: pages.length, markdown: renderReport}, null, 2)],
];
const fields = ['site-write-v1', String(generated.length), ...generated.flat()];
if (fields.some(field => typeof field !== 'string' || field.includes('\0'))) {
  throw new Error('網站產物含有不支援的 NUL 字元或非文字欄位');
}
execFileSync(process.env.CARGO || 'cargo', [
  'run', '--quiet', '--locked', '--manifest-path', 'tools/site-tools/Cargo.toml', '--bin', 'site-write',
], {cwd: projectRoot, input: fields.join('\0') + '\0', stdio: ['pipe','inherit','inherit']});
if (process.env.VERIFY_SITE_WRITE === '1') {
  const {verifySiteWrite} = await import('./migration/verify-site-write.mjs');
  await verifySiteWrite(projectRoot, generated);
}
console.log('Built ' + pages.length + ' HTML pages at ' + base + ' (Rust layout, search index and writer)');
