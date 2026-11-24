import { test, expect } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import { projectRoot } from './support/course.mjs';

function nativeAnchors(groups) {
  const fields = ['anchors-v1', String(groups.length), ...groups.flatMap(labels => [String(labels.length), ...labels])];
  const output = execFileSync(process.env.CARGO || 'cargo', [
    'run', '--quiet', '--locked', '--manifest-path', 'tools/site-render/Cargo.toml',
  ], { cwd: projectRoot, input: fields.join('\0') + '\0', encoding:'utf8', maxBuffer:1024*1024 });
  const result = output.split('\0');
  if (result.pop() !== '' || result.shift() !== 'anchors-v1' || result.shift() !== String(groups.length)) {
    throw new Error('Rust 錨點回傳格式錯誤');
  }
  let cursor=0;
  const actual=groups.map(labels=>{
    if (Number(result[cursor++]) !== labels.length) throw new Error('錨點數量錯誤');
    const ids=result.slice(cursor,cursor+labels.length);
    cursor+=labels.length;
    return ids;
  });
  if (cursor!==result.length) throw new Error('錨點含有額外欄位');
  return actual;
}

function original(labels) {
  const seen = new Map();
  return labels.map(label => {
    const stem = label.normalize('NFKC').toLowerCase().replace(/[^\p{L}\p{N}]+/gu,'-').replace(/^-|-$/g,'') || 'section';
    const n = (seen.get(stem) ?? 0) + 1;
    seen.set(stem,n);
    return 's-' + stem + (n > 1 ? '-' + n : '');
  });
}
test('native anchors match legacy Unicode and duplicate rules', () => {
  const groups = [
    ['ＡＢＣ ①','abc 1','繁體中文—Rust','🦀','','É','e\u0301','ΟΣ','İ','a\u0345b','Ⅳ','𐐀'],
    ['a','a','a-2','a','a-2','---','😄','C++','C#','你好 世界','\tHello\nWorld'],
    ['مرحبا','हिन्दी','カタカナ','ｶﾀｶﾅ','中文','中文','０９','ﬃ','Ⅷ'],
    [],
  ];
  expect(nativeAnchors(groups)).toEqual(groups.map(original));
});
