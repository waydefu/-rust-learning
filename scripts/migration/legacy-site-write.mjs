// Frozen Node writer reference. The write/copy sequence comes from the
// pre-migration scripts/build.mjs. The adapter adds a deterministic CLI boundary.
import { cp, mkdir, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { readFileSync } from 'node:fs';

export function decode(input) {
  const invalid = () => { throw new Error('無效的網站產物資料'); };
  const fields = input.split('\0');
  if (fields.pop() !== '' || fields.shift() !== 'site-write-v1') invalid();
  const raw = fields.shift();
  if (!/^\d+$/.test(raw ?? '') || Number(raw) > 1000 || fields.length !== 2 * Number(raw)) invalid();
  const files = [];
  const seen = new Set();
  for (let i = 0; i < fields.length; i += 2) {
    const [path, text] = fields.slice(i, i + 2);
    if (!path || path.split('/').some(part =>
      !/^[A-Za-z0-9_.-]+$/.test(part) || part.endsWith('.') || part === '.' || part === '..'
    ) || seen.has(path)) invalid();
    seen.add(path);
    files.push([path,text]);
  }
  for (const [path] of files) {
    const parts = path.split('/');
    for (let n = 1; n < parts.length; n++) if (seen.has(parts.slice(0,n).join('/'))) invalid();
  }
  return files;
}

export async function legacyWrite(root, files) {
  const dist = join(root,'dist');
  await rm(dist, { recursive: true, force: true });
  await mkdir(dist, { recursive: true });
  await cp(join(root, 'src/assets'), join(dist, 'assets'), { recursive: true });
  await Promise.all(files.map(async ([path, contents]) => {
    const target = join(dist, path);
    await mkdir(dirname(target), { recursive: true });
    await writeFile(target, contents);
  }));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    await legacyWrite(process.cwd(), decode(readFileSync(0, 'utf8')));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
