export const page = JSON.parse(document.querySelector('#page-data').textContent);
export const chapterPaths = new Map(page.chapterPaths);
export const storage = {
  get(key) {
    try { return localStorage.getItem(`rust-learning-${key}`); } catch { return null; }
  },
  set(key, value) {
    try { localStorage.setItem(`rust-learning-${key}`, value); return true; } catch { return false; }
  },
};

export const root = document.documentElement;
