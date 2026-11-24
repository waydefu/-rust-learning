import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { projectRoot, catalog } from '../src/course.mjs';
import { equivalentMarkup } from '../src/native-render.mjs';

test('native render compatibility comparison protects code and link destinations', () => {
  expect(equivalentMarkup('<p>Rust</p>\n<p>教材</p>', '<p>Rust</p><p>教材</p>')).toBe(true);
  expect(equivalentMarkup('<p>a <strong>b</strong></p>', '<p>a<strong>b</strong></p>')).toBe(false);
  expect(equivalentMarkup('<pre><code>a\n b</code></pre>', '<pre><code>a b</code></pre>')).toBe(false);
  expect(equivalentMarkup('<a href="/x/">x</a>', '<a href="/y/">x</a>')).toBe(false);
  expect(equivalentMarkup('<p>教材</p>', '<p>教學</p>')).toBe(false);
});

test('every built route uses Rust layout and Markdown selection is auditable', async () => {
  const report = JSON.parse(await readFile(join(projectRoot,'dist/build-render-report.json'),'utf8'));
  expect(report.layout).toBe('rust');
  expect(report.pages).toBe(34);
  expect(report.markdown.map(x => x.slug)).toEqual(catalog.map(x => x.slug));
  expect(report.markdown.some(x => x.renderer === 'rust')).toBe(true);
  for (const item of report.markdown) {
    expect(['rust','legacy']).toContain(item.renderer);
    if (item.renderer === 'legacy') expect(item.reason.length).toBeGreaterThan(0);
  }
});
