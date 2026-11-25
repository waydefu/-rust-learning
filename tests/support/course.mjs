import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

export const projectRoot = fileURLToPath(new URL('../../', import.meta.url));
export const catalog = JSON.parse(await readFile(new URL('../../content/catalog.json', import.meta.url), 'utf8'));
