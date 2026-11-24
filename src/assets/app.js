// Browser entry: feature modules own their own state and event handlers.
import { initNavigation, initToc } from './modules/navigation.js';
import { initProgress } from './modules/progress.js';
import { initReader } from './modules/reader.js';
import { initMotion } from './modules/motion.js';
import { initConceptLabs } from './modules/concept-labs.js';

initNavigation();
initProgress();
initToc();
initReader();
initMotion();
initConceptLabs();
