// Explicit recording corpus for the isolated writer; never runs against the
// working checkout. The full current site is independently compared in CI.
const packet = (...fields) => fields.join('\0') + '\0';
const request = files => packet('site-write-v1', String(files.length), ...files.flat());
const assets = {'src/assets/site.css':'body { color: black; }\n',
  'src/assets/nested/圖解.svg':'<svg xmlns="http://www.w3.org/2000/svg"/>\n'};
const cases = [
  ['full-miniature', request([['index.html','<h1>Rust 學習工坊🦀</h1>\n'],
    ['chapters/ownership/index.html','<p>借用</p>\n'], ['assets/search-index.json','[{"id":"04"}]'],
    ['.nojekyll','']])],
  ['empty-generated-file', request([['index.html','']])],
  ['nested-pages', request([['guide/outline/index.html','第一章\n第二章\n'],['404.html','找不到']])],
  ['zero-generated', request([])],
  ['replace-old-output', request([['index.html','新版']]), {'dist/index.html':'舊版','dist/stale/old.html':'stale'}],
  ['traversal', request([['../outside.txt','bad']])],
  ['absolute', request([['/outside.txt','bad']])],
  ['duplicate', request([['index.html','x'],['index.html','y']])],
  ['path-conflict', request([['a','x'],['a/b','y']])],
  ['backslash', request([['a\\b','x']])],
  ['truncated', packet('site-write-v1','1','index.html')],
  ['wrong-version', packet('old','0')],
  ['extra-fields', packet('site-write-v1','0','extra')],
  ['non-ascii-path', request([['教材.html','x']])],
];
for (const [name, stdin, extra = {}] of cases) {
  console.log(JSON.stringify({name, stdin, files:{...assets,...extra}}));
}
