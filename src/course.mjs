import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import MarkdownIt from 'markdown-it';
import hljs from 'highlight.js/lib/core';
import rust from 'highlight.js/lib/languages/rust';
import bash from 'highlight.js/lib/languages/bash';
import toml from 'highlight.js/lib/languages/ini';
import { icon } from './icons.mjs';

hljs.registerLanguage('rust', rust);
hljs.registerLanguage('bash', bash);
hljs.registerLanguage('toml', toml);

export const projectRoot = fileURLToPath(new URL('../', import.meta.url));
export const catalog = JSON.parse(await readFile(new URL('../content/catalog.json', import.meta.url), 'utf8'));
export const chapters = catalog.filter(item => /^\d{2}$/.test(item.id));
export const resources = catalog.filter(item => /^A\d{2}$/.test(item.id));
export const stages = [
  { id: 'fundamentals', title: '入門與所有權', icon: 'terminal', description: '從第一支程式出發，理解變數、控制流程與記憶體。' },
  { id: 'modeling', title: '資料建模與錯誤', icon: 'box', description: '用型別描述狀態，組織程式並處理可預期的失敗。' },
  { id: 'engineering', title: '抽象與工程實作', icon: 'branch', description: '透過 Trait、測試與實作，寫出可重用、可維護的程式。' },
  { id: 'systems', title: '並行與系統開發', icon: 'cpu', description: '掌握資源共享與排程，完成一個多執行緒伺服器。' },
].map(stage => ({ ...stage, chapters: chapters.filter(item => item.group === stage.id) }));

export const escape = value => String(value).replace(/[&<>"']/g, character => ({
  '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;',
})[character]);

export const md = new MarkdownIt({ html: false, linkify: false });

// The internal kind selects a color, not the reader-facing heading.
// The HTML comment is parsed as text (raw HTML stays disabled), then removed.
export const calloutTypes = Object.freeze(['context', 'analogy', 'walkthrough', 'warning', 'practice']);
md.core.ruler.after('inline', 'lesson_callouts', state => {
  const tokens = state.tokens;
  for (let i = 0; i < tokens.length; i++) {
    const open = tokens[i];
    if (open.type !== 'blockquote_open' || tokens[i + 1]?.type !== 'paragraph_open') continue;
    const label = tokens[i + 2];
    if (label?.type !== 'inline' || tokens[i + 3]?.type !== 'paragraph_close') continue;
    const match = /^\*\*([^*]+)\*\*\s+<!-- callout:([a-z]+) -->$/.exec(label.content.trim());
    const type = match && calloutTypes.includes(match[2]) ? match[2] : null;
    if (!type) continue;
    const close = tokens.findIndex((token, j) => j > i && token.type === 'blockquote_close' && token.level === open.level);
    if (close === -1 || close <= i + 4) continue;
    label.content = '**' + match[1] + '**';
    label.children = [];
    state.md.inline.parse(label.content, state.md, state.env, label.children);
    open.tag = 'div';
    tokens[close].tag = 'div';
    open.attrSet('class', 'lesson-callout callout-' + type);
    open.attrSet('data-callout', type);
    open.attrSet('role', 'note');
    tokens[i + 1].attrSet('class', 'callout-title');
  }
});
export function highlightCode(token) {
  const [language] = token.info.trim().split(/\s+/);
  return hljs.getLanguage(language)
    ? hljs.highlight(token.content, { language, ignoreIllegals: true }).value
    : escape(token.content);
}
md.renderer.rules.fence = (tokens, index) => {
  const token = tokens[index];
  const [language, mode] = token.info.trim().split(/\s+/);
  const name = { rust: 'Rust', bash: 'Shell', toml: 'TOML', text: '文字' }[language] ?? '程式碼';
  const note = { fragment: '片段', continuation: '接續範例', test: '測試片段', project: '專案範例', error: '編譯反例', exercise: '練習反例', run: '' }[mode];
  const highlighted = highlightCode(token);
  return `<div class="code-block"><div class="code-header"><span>${name}${note ? ` <small>${note}</small>` : ''}</span><button type="button" class="copy-button" data-copy hidden>${icon('copy', 16)}<span>複製</span></button></div><pre tabindex="0" aria-label="${name}程式碼"><code class="language-${escape(language)}">${highlighted}</code></pre></div>\n`;
};

const plain = token => (token.children ?? []).map(child => {
  if (child.type === 'image') return child.content;
  return child.type === 'text' || child.type === 'code_inline' ? child.content : '';
}).join('');

export async function readLesson(entry) {
  const source = await readFile(new URL(`../content/${entry.slug}.md`, import.meta.url), 'utf8');
  const tokens = md.parse(source, {});
  const title = tokens[1];
  if (tokens[0]?.tag !== 'h1' || !title || tokens.filter(token => token.type === 'heading_open' && token.tag === 'h1').length !== 1) {
    throw new Error(`${entry.slug}.md 必須以單一 H1 開頭`);
  }
  const headings = [];
  const seen = new Map();
  for (let index = 3; index < tokens.length; index++) {
    const token = tokens[index];
    if (token.type !== 'heading_open') continue;
    const label = plain(tokens[index + 1]);
    const stem = label.normalize('NFKC').toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '-').replace(/^-|-$/g, '') || 'section';
    const count = (seen.get(stem) ?? 0) + 1;
    seen.set(stem, count);
    const id = `s-${stem}${count > 1 ? `-${count}` : ''}`;
    token.attrSet('id', id);
    token.attrSet('tabindex', '-1');
    headings.push({ id, label, level: Number(token.tag.slice(1)), tokenIndex: index });
  }
  const fullText = tokens.filter(token => token.type === 'inline' || token.type === 'fence')
    .map(token => token.type === 'fence' ? token.content : plain(token)).join(' ').replace(/\s+/g, ' ');
  return { entry, source, title: plain(title), tokens: tokens.slice(3), headings, fullText, searchTitle: title.content };
}

export function renderTokens(tokens) {
  return md.renderer.render(tokens, md.options, {});
}
