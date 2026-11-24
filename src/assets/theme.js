try {
  const savedTheme = localStorage.getItem('rust-learning-theme');
  if (savedTheme === 'light' || savedTheme === 'dark') document.documentElement.dataset.theme = savedTheme;
} catch {}
