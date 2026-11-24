import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join } from 'node:path';
import { projectRoot } from '../src/course.mjs';

await import('./build.mjs');
const base = (process.env.BASE_PATH || '/').replace(/\/?$/, '/');
const mime = {
  '.html':'text/html; charset=utf-8',
  '.css':'text/css; charset=utf-8',
  '.js':'text/javascript; charset=utf-8',
  '.svg':'image/svg+xml',
  '.json':'application/json; charset=utf-8',
  '.txt':'text/plain; charset=utf-8',
};

createServer(async (request,response) => {
  try {
    const raw = new URL(request.url, 'http://localhost').pathname;
    if (!raw.startsWith(base)) throw new Error('Outside deployment path');
    const path = decodeURIComponent(raw.slice(base.length));
    if (path.includes('\\') || path.split('/').some(segment => ['.','..'].includes(segment))) throw new Error('Invalid path');
    const extension = extname(path);
    const file = extension ? path : join(path, 'index.html');
    const bytes = await readFile(join(projectRoot, 'dist', file));
    if (!extension && !raw.endsWith('/')) {
      response.writeHead(308, { Location: raw + '/' });
      response.end();
      return;
    }
    response.writeHead(200, {'Content-Type': mime[extname(file)] ?? 'application/octet-stream'}).end(bytes);
  } catch {
    response.writeHead(404, {'Content-Type': mime['.html']}).end(await readFile(join(projectRoot,'dist','404.html')));
  }
}).listen(Number(process.env.PORT ?? 4173), process.env.HOST ?? '0.0.0.0', () => {
  console.log(`Rust 學習工坊 http://localhost:${process.env.PORT ?? 4173}${base}`);
});
