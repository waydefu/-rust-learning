import { page, root, storage } from './context.js';

export function initNavigation() {
  const preference = matchMedia('(prefers-color-scheme: dark)');
  const themeButton = document.querySelector('#theme-toggle');
  function updateThemeButton() {
    const isDark = root.dataset.theme ? root.dataset.theme === 'dark' : preference.matches;
    themeButton.setAttribute('aria-label', isDark ? '切換淺色模式' : '切換深色模式');
    themeButton.title = themeButton.getAttribute('aria-label');
  }
  themeButton.hidden = false;
  updateThemeButton();
  themeButton.addEventListener('click', () => {
    const isDark = root.dataset.theme ? root.dataset.theme === 'dark' : preference.matches;
    root.dataset.theme = isDark ? 'light' : 'dark';
    storage.set('theme', root.dataset.theme);
    updateThemeButton();
  });
  preference.addEventListener('change', updateThemeButton);

  const menu = document.querySelector('#menu-dialog');
  const menuOpen = document.querySelector('#menu-open');
  menuOpen.hidden = false;
  menuOpen.addEventListener('click', () => menu.showModal());
  matchMedia('(min-width: 811px)').addEventListener('change', event => {
    if (event.matches && menu.open) menu.close();
  });

  const search = document.querySelector('#search-dialog');
  const searchButton = document.querySelector('#search-open');
  const searchInput = document.querySelector('#search-input');
  const searchStatus = document.querySelector('#search-status');
  const searchResults = document.querySelector('#search-results');
  const shortcut = document.querySelector('[data-shortcut]');
  shortcut.textContent = /Mac|iPhone|iPad/.test(navigator.platform) ? '⌘ K' : 'Ctrl K';
  searchButton.closest(".search-wrap").hidden = false;
  let indexRequest;
  let searchRevision = 0;
  const normalize = value => value.normalize('NFKC').toLowerCase();
  function loadIndex() {
    indexRequest ??= fetch(`${page.base}assets/search-index.json`).then(response => {
      if (!response.ok) throw new Error('無法下載搜尋索引');
      return response.json();
    }).catch(error => {
      indexRequest = undefined;
      throw error;
    });
    return indexRequest;
  }

  function openSearch() {
    if (menu.open) menu.close();
    if (!search.open) search.showModal();
    searchInput.focus();
    renderSearch();
  }
  searchButton.addEventListener('click', openSearch);
  search.addEventListener('click', event => {
    if (event.target !== search) return;
    const bounds = search.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) search.close();
  });
  document.addEventListener('keydown', event => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      openSearch();
    }
  });
  searchInput.addEventListener('input', renderSearch);
  searchInput.addEventListener('keydown', event => {
    if (event.key !== 'ArrowDown') return;
    const first = searchResults.querySelector('a');
    if (first) {
      event.preventDefault();
      first.focus();
    }
  });
  searchResults.addEventListener('keydown', event => {
    if (!['ArrowUp', 'ArrowDown'].includes(event.key) || !event.target.matches('a')) return;
    const links = Array.from(searchResults.querySelectorAll('a'));
    const next = links[links.indexOf(event.target) + (event.key === 'ArrowUp' ? -1 : 1)];
    if (next || event.key === 'ArrowUp') {
      event.preventDefault();
      (next ?? searchInput).focus();
    }
  });

  async function renderSearch() {
    const revision = ++searchRevision;
    const query = normalize(searchInput.value.trim());
    searchResults.replaceChildren();
    if (!query) {
      searchStatus.textContent = '輸入章名、概念、程式碼或編譯錯誤。';
      return;
    }
    searchStatus.textContent = '搜尋中…';
    try {
      const entries = await loadIndex();
      if (revision !== searchRevision) return;
      const terms = query.split(/\s+/).filter(Boolean);
      const found = entries.flatMap(entry => {
        const title = normalize(entry.title);
        const body = normalize(`${entry.id} ${entry.description} ${entry.text}`);
        const heading = entry.headings.find(section => terms.every(term => normalize(section.label).includes(term)));
        if (!terms.every(term => title.includes(term) || body.includes(term))) return [];
        const score = title.includes(query) ? 4 : heading ? 3 : body.includes(query) ? 2 : 1;
        return [{ entry, score, heading: score === 3 ? heading : null }];
      }).sort((a, b) => b.score - a.score);
      searchStatus.textContent = found.length ? `找到 ${found.length} 筆${found.length > 12 ? '，顯示前 12 筆' : ''}` : '沒有符合的結果，試試較短的詞。';
      for (const { entry, heading } of found.slice(0, 12)) {
        const li = document.createElement('li');
        const link = document.createElement('a');
        link.href = entry.href + (heading ? '#' + encodeURIComponent(heading.id) : '');
        const title = document.createElement('strong');
        title.textContent = /^\d+$/.test(entry.id) ? `${entry.id}　${entry.title}` : entry.title;
        const description = document.createElement('small');
        description.textContent = heading?.label ?? entry.description;
        link.append(title, description);
        li.append(link);
        searchResults.append(li);
      }
    } catch {
      if (revision === searchRevision) searchStatus.textContent = '無法載入索引。請再試一次，或從課程導覽開啟教材。';
    }
  }
}

export function initToc() {
  for (const link of document.querySelectorAll('.mobile-toc a[href^="#"]')) {
    link.addEventListener('click', () => link.closest('details').open = false);
  }
  const tocLinks = Array.from(document.querySelectorAll('.desktop-toc a[href^="#"]'));
  const sections = tocLinks.map(link => document.getElementById(link.getAttribute('href').slice(1))).filter(Boolean);
  if (sections.length) {
    let active;
    let frame;
    function getHashSection() {
      try {
        const id = decodeURIComponent(location.hash.slice(1));
        return sections.find(section => {
          if (section.id !== id) return false;
          const top = section.getBoundingClientRect().top;
          return top >= 0 && top < innerHeight - 32;
        });
      } catch { return undefined; }
    }
    const updateLocation = () => {
      const pinned = getHashSection();
      const maxScroll = document.documentElement.scrollHeight - innerHeight;
      const atBottom = maxScroll > 0 && scrollY >= maxScroll - 2;
      const underHeader = sections.filter(section => section.getBoundingClientRect().top <= 142).at(-1);
      const next = pinned ?? (atBottom ? sections.at(-1) : underHeader);
      if (next !== active) {
        active = next;
        for (const link of tocLinks) {
          if (link.getAttribute('href') === `#${active?.id}`) link.setAttribute('aria-current', 'location');
          else link.removeAttribute('aria-current');
        }
        if (page.chapter) storage.set(`last-section-${page.chapter}`, active?.id ?? '');
      }
    };
    window.addEventListener('scroll', () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(updateLocation);
    }, { passive: true });
    window.addEventListener('hashchange', () => requestAnimationFrame(updateLocation));
    window.addEventListener('pageshow', () => requestAnimationFrame(updateLocation));
    updateLocation();
  }
}
