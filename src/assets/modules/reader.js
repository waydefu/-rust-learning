import { page } from './context.js';
import { motionFeedback } from './motion.js';

export function initReader() {
  for (const button of document.querySelectorAll('[data-copy]')) {
    button.hidden = false;
    let restore;
    button.addEventListener('click', async () => {
      const label = button.querySelector('span');
      const code = button.closest('.code-block').querySelector('code');
      try {
        await navigator.clipboard.writeText(code.textContent.replace(/\n$/, ''));
        label.textContent = '已複製';
        motionFeedback(button, [{transform:'scale(.96)'},{transform:'none'}]);
      } catch {
        const selection = window.getSelection();
        if (!selection) return;
        const range = document.createRange();
        range.selectNodeContents(code);
        selection.removeAllRanges();
        selection.addRange(range);
        label.textContent = '已選取，請複製';
      }
      clearTimeout(restore);
      restore = setTimeout(() => label.textContent = '複製', 2200);
    });
  }

  const lab = document.querySelector('[data-ownership]');
  if (lab) {
    const controls = lab.querySelector('[data-steps]');
    const visual = lab.querySelector('#memory-state');
    controls.hidden = false;
    controls.addEventListener('click', event => {
      const button = event.target.closest('[data-step]');
      if (!button) return;
      const step = page.ownershipSteps[Number(button.dataset.step)];
      for (const control of controls.querySelectorAll('button')) control.setAttribute('aria-pressed', String(control === button));
      visual.dataset.owner = step.owner ?? 'none';
      visual.dataset.borrowed = String(Boolean(step.borrowed));
      lab.querySelector('[data-lab-code]').textContent = step.code;
      lab.querySelector('[data-binding-a]').textContent = step.source;
      lab.querySelector('[data-binding-b]').textContent = step.target;
      lab.querySelector('[data-lab-message]').textContent = step.message;
      const owner = step.owner === 'a' ? '[data-binding-a]' : step.owner === 'b' ? '[data-binding-b]' : null;
      if (owner) motionFeedback(lab.querySelector(owner), [{transform:'translateX(-3px)'},{transform:'none'}]);
      if (step.owner) motionFeedback(lab.querySelector('.memory-bytes'), [{transform:'scale(.97)'},{transform:'none'}], 300);
    });
  }

  const quiz = document.querySelector('[data-quiz]');
  if (quiz) {
    const form = quiz.querySelector('form');
    const feedback = form.querySelector('[data-quiz-feedback]');
    const answer = quiz.querySelector('.answer');
    form.querySelector('button').hidden = false;
    form.addEventListener('change', () => {
      feedback.textContent = '';
      delete feedback.dataset.result;
    });
    form.addEventListener('submit', event => {
      event.preventDefault();
      const chosen = new FormData(form).get('answer');
      if (chosen === null) return;
      const correct = Number(chosen) === page.answer;
      feedback.dataset.result = correct ? 'correct' : 'retry';
      feedback.textContent = correct ? '答對了。下方有完整解析。' : '再試一次，或展開解析確認原因。';
      if (correct) answer.open = true;
      motionFeedback(feedback, [{transform:'translateY(3px)'},{transform:'none'}]);
    });
  }
}
