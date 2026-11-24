import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { projectRoot, catalog } from './support/course.mjs';

test('every built route uses native Rust rendering with zero fallback', async () => {
  const report = JSON.parse(await readFile(join(projectRoot,'dist/build-render-report.json'),'utf8'));
  expect(report.layout).toBe('rust');
  expect(report.pages).toBe(catalog.length+3);
  expect(report.markdown.map(x => x.slug)).toEqual(catalog.map(x => x.slug));
  expect(report.markdown.every(x=>x.renderer==='rust' && x.reason==='')).toBe(true);
  expect(report.highlight.renderer).toBe('rust');
  expect(report.highlight.total).toBeGreaterThan(0);
  expect(report.highlight.native).toBe(report.highlight.total);
  expect(report.highlight.fallback).toBe(0);
});
