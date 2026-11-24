import { readFileSync } from 'node:fs';
export const siteData = JSON.parse(readFileSync(new URL('../content/site-data.json', import.meta.url), 'utf8'));
