import { page, root } from './context.js';

const motionPreference = matchMedia('(prefers-reduced-motion: reduce)');
const feedbackAnimations = new WeakMap();
const activeAnimations = new Set();
export function motionFeedback(element, frames, duration = 220) {
  if (!element || motionPreference.matches || typeof element.animate !== 'function') return;
  feedbackAnimations.get(element)?.cancel();
  const animation = element.animate(frames, {duration, easing: 'cubic-bezier(.2,.75,.25,1)'});
  feedbackAnimations.set(element, animation);
  activeAnimations.add(animation);
  const cleanup = () => activeAnimations.delete(animation);
  animation.addEventListener('finish', cleanup, {once:true});
  animation.addEventListener('cancel', cleanup, {once:true});
}
motionPreference.addEventListener('change', event => {
  if (event.matches) for (const animation of activeAnimations) animation.cancel();
  if (event.matches) document.querySelector('.startup-motion')?.remove();
});

export function initMotion() {
  // A non-modal welcome animation once per tab session. No remote resources,
  // content hiding, focus changes, or delayed navigation; no-JS pages stay intact.
  function startupMotion() {
    if (motionPreference.matches || location.hash || !document.querySelector('.hero')) return;
    try {
      if (sessionStorage.getItem('rust-learning-startup') === '1') return;
      sessionStorage.setItem('rust-learning-startup', '1');
    } catch { return; }
    const badge = document.createElement('div');
    badge.className = 'startup-motion';
    badge.setAttribute('aria-hidden', 'true');
    const logo = document.createElement('img');
    logo.src = page.base + 'assets/favicon.svg';
    logo.alt = '';
    const copy = document.createElement('div');
    const title = document.createElement('strong');
    title.textContent = 'Rust 學習工坊';
    const sub = document.createElement('small');
    sub.textContent = 'RUST LEARNING LAB';
    copy.append(title, sub);
    badge.append(logo, copy);
    document.body.append(badge);
    const remove = () => badge.remove();
    badge.addEventListener('animationend', event => { if (event.target === badge) remove(); }, {once:false});
    setTimeout(remove, 1400);
  }
  startupMotion();


  // Reading progress reflects actual scroll, not chapter completion. Decorative;
  // no focus or announcement changes. The content is never hidden.
  if (page.chapter) {
    const indicator = document.createElement('div');
    indicator.className = 'reading-indicator';
    indicator.setAttribute('aria-hidden', 'true');
    const fill = document.createElement('span');
    indicator.append(fill);
    document.body.append(indicator);
    let pending = false;
    const update = () => {
      pending = false;
      const range = Math.max(0, root.scrollHeight - innerHeight);
      const fraction = range ? Math.min(1, Math.max(0, scrollY / range)) : 1;
      fill.style.transform = 'scaleX(' + fraction + ')';
    };
    const schedule = () => {
      if (!pending) { pending = true; requestAnimationFrame(update); }
    };
    addEventListener('scroll', schedule, {passive:true});
    addEventListener('resize', schedule, {passive:true});
    addEventListener('pageshow', schedule);
    if ('ResizeObserver' in window) new ResizeObserver(schedule).observe(document.body);
    update();
  }
}
