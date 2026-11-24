import { test, expect } from '@playwright/test';
import { nativeAnchors } from '../src/native-render.mjs';

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
