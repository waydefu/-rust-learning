import { page, chapterPaths, storage } from './context.js';

export function initProgress() {
  const progress = document.querySelector('[data-progress]');
  const complete = document.querySelector('#complete');
  const completed = new Set([...chapterPaths.keys()].filter(id => storage.get(`completed-${id}`) === '1'));
  if (page.chapter) storage.set('last-chapter', page.chapter);

  function renderProgress() {
    document.querySelectorAll('[data-nav-chapter]').forEach(link => {
      link.querySelector('.nav-done').hidden = !completed.has(link.dataset.navChapter);
    });
    document.querySelectorAll('[data-card-chapter]').forEach(card => {
      card.querySelector('.card-done').hidden = !completed.has(card.dataset.cardChapter);
    });
    if (complete) {
      const done = completed.has(page.chapter);
      complete.setAttribute('aria-pressed', String(done));
      complete.querySelector('span').textContent = done ? '取消完成' : '標記完成';
    }
    if (!progress) return;
    progress.querySelector('progress').value = completed.size;
    progress.querySelector('[data-progress-count]').textContent = `${completed.size} / ${chapterPaths.size} 章`;
    progress.querySelector('[data-progress-percent]').textContent = `${Math.round(completed.size * 100 / chapterPaths.size)}%`;
    const lastChapter = storage.get('last-chapter');
    if (chapterPaths.has(lastChapter)) {
      const path = chapterPaths.get(lastChapter);
      const section = storage.get(`last-section-${lastChapter}`);
      const resumeHref = `${page.base}${path}/${section ? '#' + encodeURIComponent(section) : ''}`;
      for (const link of [progress.querySelector('[data-resume]'), document.querySelector('[data-start]')]) {
        link.href = resumeHref;
        link.textContent = link.matches('[data-start]') ? '繼續學習' : `繼續第 ${Number(lastChapter)} 章`;
      }
    }
  }
  renderProgress();
  if (complete) {
    complete.hidden = false;
    complete.addEventListener('click', () => {
      const done = !completed.has(page.chapter);
      const persisted = storage.set(`completed-${page.chapter}`, done ? '1' : '0');
      if (done) completed.add(page.chapter);
      else completed.delete(page.chapter);
      renderProgress();
      document.querySelector('[data-completion-status]').textContent = !persisted
        ? '目前瀏覽器未允許儲存，重新載入後可能遺失進度。'
        : done ? '本章已標記完成，進度已儲存。' : '已取消本章的完成標記。';
    });
  }
  window.addEventListener('storage', event => {
    if (!event.key?.startsWith('rust-learning-completed-')) return;
    const id = event.key.slice('rust-learning-completed-'.length);
    if (!chapterPaths.has(id)) return;
    if (event.newValue === '1') completed.add(id);
    else completed.delete(id);
    renderProgress();
  });
}
