// Compare the actual site tree against the frozen Node reference in isolation.
import { cp, mkdtemp, readdir, readFile, rm, mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { legacyWrite } from './legacy-site-write.mjs';

async function manifest(root, folder = '') {
  const files = new Map();
  for (const entry of await readdir(join(root, folder), {withFileTypes: true})) {
    const path = folder ? folder + '/' + entry.name : entry.name;
    if (entry.isDirectory()) {
      for (const [key, value] of await manifest(root, path)) files.set(key, value);
    } else if (entry.isFile()) files.set(path, await readFile(join(root,path)));
    else throw new Error('不支援的網站產物：' + path);
  }
  return files;
}

export async function verifySiteWrite(projectRoot, files) {
  const scratch = await mkdtemp(join(tmpdir(), 'rust-site-reference-'));
  try {
    await mkdir(join(scratch,'src'), {recursive:true});
    await cp(join(projectRoot,'src/assets'), join(scratch,'src/assets'), {recursive:true});
    await legacyWrite(scratch, files);
    const expected = await manifest(join(scratch,'dist'));
    const actual = await manifest(join(projectRoot,'dist'));
    if (actual.size !== expected.size) throw new Error('Rust 寫入器檔案數不相容');
    for (const [path, bytes] of expected) {
      if (!actual.get(path)?.equals(bytes)) throw new Error('Rust 寫入器檔案內容不相容：' + path);
    }
    console.log('Rust 寫入器逐位元組比對：' + actual.size + ' 份檔案通過');
  } finally { await rm(scratch, {recursive:true,force:true}); }
}
