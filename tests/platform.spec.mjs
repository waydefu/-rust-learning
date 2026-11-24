import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { stat } from 'node:fs/promises';
import { join } from 'node:path';
import { projectRoot } from '../src/course.mjs';

const origin = 'https://rust-learning.test';
const base = '/-rust-learning/';
const address = (path = '') => origin + base + path;

test.beforeEach(async ({page}) => {
  // Serve the Pages output in the browser without requiring a bound local port.
  // For a live or local hosted test, set TEST_ORIGIN and use its same base path.
  if (process.env.TEST_ORIGIN) return;
  await page.route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin !== origin || !url.pathname.startsWith(base)) return route.abort();
    const path = decodeURIComponent(url.pathname.slice(base.length));
    const file = join(projectRoot, 'dist', path, url.pathname.endsWith('/') ? 'index.html' : '');
    if ((await stat(file).catch(() => null))?.isFile()) return route.fulfill({path:file});
    return route.fulfill({status:404, path:join(projectRoot,'dist','404.html')});
  });
});

const target = path => process.env.TEST_ORIGIN ? process.env.TEST_ORIGIN.replace(/\/$/, '') + base + (path ?? '') : address(path);

test('home to chapter to next chapter and deep reload', async ({page}) => {
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto(target());
  await expect(page).toHaveTitle('Rust 學習工坊');
  await expect(page.getByRole('heading', { name:/從第一行程式，\s*到真正理解 Rust。/ })).toBeVisible();
  await expect(page.locator('.hero img')).toHaveJSProperty('complete', true);
  await expect(page.locator('.hero img')).toHaveJSProperty('naturalWidth', 448);
  await expect(page.locator('[data-card-chapter]')).toHaveCount(21);
  await page.getByRole('link', {name:'開始學習'}).click();
  await expect(page).toHaveURL(/\/chapters\/getting-started\/$/);
  const pagination = page.getByRole('navigation', {name:'相鄰章節'});
  await pagination.getByRole('link', {name:/下一章.*猜數字遊戲/}).click();
  await page.reload();
  await expect(page.getByRole('heading', {level:1})).toContainText('猜數字遊戲');
  expect(errors).toEqual([]);
});

test('search finds Chinese concepts and named diagnostics with keyboard navigation', async ({page}) => {
  await page.goto(target());
  await page.keyboard.press('Control+k');
  await expect(page.getByRole('dialog', {name:'搜尋教材'})).toBeVisible();
  const field = page.getByRole('searchbox', {name:'搜尋詞'});
  await field.fill('找不到的中文測試字串');
  await expect(page.locator('#search-status')).toContainText('沒有符合');
  await field.fill('所有權');
  await expect(page.locator('#search-results a').first()).toContainText('所有權與借用');
  await field.fill('E0382');
  await expect(page.locator('#search-results a').first()).toBeVisible();
  await field.press('ArrowDown');
  await expect(page.locator('#search-results a').first()).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', {level:1})).toContainText(/所有權|診斷/);
});

test('ownership states, quiz feedback, clipboard, theme and progress persist', async ({page, context}) => {
  await context.grantPermissions(['clipboard-read','clipboard-write']);
  await page.goto(target('chapters/ownership/'));
  const lab = page.locator('[data-ownership]');
  await lab.getByRole('button', {name:'2 移動'}).click();
  await expect(lab.locator('#memory-state')).toHaveAttribute('data-owner','b');
  await lab.getByRole('button', {name:'3 借用'}).click();
  await expect(lab.locator('#memory-state')).toHaveAttribute('data-borrowed','true');
  await lab.getByRole('button', {name:'4 釋放'}).click();
  await expect(lab.locator('#memory-state')).toHaveAttribute('data-owner','none');
  await expect(lab.getByText(/drop 消耗 b/)).toBeVisible();
  const code = page.locator('.code-block').first();
  await code.getByRole('button', {name:'複製'}).click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toContain('String::from("hello")');

  const quiz = page.locator('.quiz');
  await quiz.getByRole('radio').first().check();
  await quiz.getByRole('button', {name:'檢查答案'}).click();
  await expect(quiz.getByRole('status')).toContainText('再試一次');
  await quiz.getByRole('radio').nth(1).check();
  await quiz.getByRole('button', {name:'檢查答案'}).click();
  await expect(quiz.getByRole('status')).toContainText('答對了');
  await expect(quiz.locator('details')).toHaveAttribute('open', '');

  await page.locator('.desktop-toc a[href="#quick-check"]').click();
  await page.getByRole('button', {name:'標記完成'}).click();
  await page.getByRole('button', {name:'切換深色模式'}).click();
  await page.reload();
  await expect(page.getByRole('button', {name:'取消完成'})).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('html')).toHaveAttribute('data-theme','dark');
  await page.getByRole('link', {name:/Rust 學習工坊/}).click();
  await expect(page.locator('progress')).toHaveJSProperty('value',1);
  await expect(page.getByRole('link', {name:'繼續第 4 章'})).toHaveAttribute('href', /chapters\/ownership\/#quick-check$/);
});

test('mobile drawer, linked headings and sideways code scroll at 320px', async ({page}) => {
  await page.setViewportSize({width:320,height:790});
  await page.goto(target());
  await expect(page.locator('.sidebar')).toBeHidden();
  await page.getByRole('button',{name:'開啟課程導覽'}).click();
  const menu = page.getByRole('dialog',{name:'課程導覽'});
  await expect(menu.getByRole('link',{name:/所有權與借用/})).toBeVisible();
  await menu.getByRole('link',{name:/所有權與借用/}).click();
  await page.locator('.mobile-toc summary').click();
  const section = page.getByRole('link',{name:/懸空參考到底危險在哪/});
  await section.click();
  await expect(page.locator('.mobile-toc')).not.toHaveAttribute('open', '');
  await expect(page).toHaveURL(/#/);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(320);
  expect(await page.locator('pre').evaluateAll(blocks => blocks.some(block => block.scrollWidth > block.clientWidth))).toBe(true);
});

test('WCAG A/AA automated rules on desktop home and learner view', async ({page}) => {
  const tags = ['wcag2a','wcag2aa','wcag21a','wcag21aa'];
  await page.goto(target());
  expect((await new AxeBuilder({page}).withTags(tags).analyze()).violations).toEqual([]);
  await page.setViewportSize({width:390,height:830});
  await page.goto(target('chapters/ownership/'));
  await page.getByRole('button', {name:'切換深色模式'}).click();
  expect((await new AxeBuilder({page}).withTags(tags).analyze()).violations).toEqual([]);
});

test('lesson and quiz explanation are readable without JavaScript', async ({browser}) => {
  const noJS = await browser.newContext({javaScriptEnabled:false});
  try {
    const page = await noJS.newPage();
    const file = join(projectRoot,'dist','chapters','ownership','index.html');
    await page.route(address('chapters/ownership/'), route => route.fulfill({path:file}));
    await page.goto(address('chapters/ownership/'));
    await expect(page.getByRole('heading',{level:1})).toContainText('所有權');
    await expect(page.getByText(/drop 消耗 b/)).toBeVisible();
    await page.getByText('查看完整解析',{exact:true}).click();
    await expect(page.getByText('仍是同一個配置，由 b 負責釋放',{exact:true}).last()).toBeVisible();
  } finally {
    await noJS.close();
  }
});
