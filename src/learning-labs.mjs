import { escape } from './course.mjs';
import { icon } from './icons.mjs';
import { siteData } from './site-data.mjs';

export const ownershipSteps = siteData.ownershipSteps;

export function ownershipLab() {
  return `<section class="lab" data-ownership>
  <p class="lab-kicker">${icon('layers', 17)} 互動記憶體圖</p>
  <h2 id="ownership-lab" tabindex="-1">跟著 String 的所有權走</h2>
  <p>每一步都先猜誰負責釋放緩衝區，再切換狀態。圖上只呈現語意上的綁定與配置。</p>
  <div class="lab-controls" data-steps hidden>${['建立', '移動', '借用', '釋放'].map((label, index) => `<button type="button" data-step="${index}" aria-controls="memory-state" aria-pressed="${index === 0}">${index + 1} ${label}</button>`).join('')}</div>
  <div class="lab-visual" id="memory-state" data-owner="a">
    <code data-lab-code>${escape(ownershipSteps[0].code)}</code>
    <div class="memory-grid" aria-hidden="true">
      <div><span class="memory-label">綁定</span><div class="memory-node" data-binding-a>a: String</div><div class="memory-node" data-binding-b>b 尚未宣告</div></div>
      <svg viewBox="0 0 86 114" preserveAspectRatio="none" focusable="false"><defs><marker id="memory-arrow" markerWidth="7" markerHeight="8" refX="5" refY="4" orient="auto"><path d="m0 1 5 3-5 3" fill="none" stroke="context-stroke"/></marker></defs><path data-line-a d="M1 43C38 43 38 66 82 66" marker-end="url(#memory-arrow)" /><path data-line-b d="M1 90C38 90 38 66 82 66" marker-end="url(#memory-arrow)" /></svg>
      <div><span class="memory-label">堆積緩衝區</span><div class="memory-bytes"><span>R</span><span>u</span><span>s</span><span>t</span></div></div>
    </div>
    <p class="lab-message" data-lab-message aria-live="polite">${ownershipSteps[0].message}</p>
  </div>
  <noscript><ol>${ownershipSteps.map(step => `<li><code>${escape(step.code)}</code> ${step.message}</li>`).join('')}</ol></noscript>
</section>`;
}

export const flows = siteData.flows;

export function conceptFlow(slug) {
  const flow = flows[slug];
  if (!flow) return '';
  return `<figure class="concept-flow"><figcaption>${flow.title}</figcaption><ol>${flow.nodes.map((node, index) => `<li>${index ? icon('chevron', 16) : ''}<span>${escape(node)}</span></li>`).join('')}</ol><p>${escape(flow.caption)}</p></figure>`;
}

export const quizzes = siteData.quizzes;

export function quiz(slug) {
  const question = quizzes[slug];
  if (!question) return '';
  return `<section class="quiz" data-quiz="${slug}">
    <p class="lab-kicker">${icon('target', 18)} 自我測驗</p>
    <h2 id="quick-check" tabindex="-1">確認理解</h2>
    <form><fieldset><legend>${escape(question.prompt)}</legend>
    ${question.choices.map((choice, index) => `<label><input type="radio" name="answer" value="${index}" required><span>${escape(choice)}</span></label>`).join('')}
    </fieldset><button type="submit" class="button primary" hidden>檢查答案</button>
    <p data-quiz-feedback role="status"></p></form>
    <details class="answer"><summary>查看完整解析</summary><p><strong>${escape(question.choices[question.answer])}</strong></p><p>${escape(question.explanation)}</p></details>
  </section>`;
}
