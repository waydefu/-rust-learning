import { spawnSync } from 'node:child_process';
import { projectRoot } from '../src/course.mjs';

await import('./build.mjs');

// The same Rust server used by the CLI serves the real static output.
// Cargo inherits the terminal so Ctrl+C ends the preview normally.
const result = spawnSync(process.env.CARGO || 'cargo', [
  'run', '--quiet', '--locked', '--manifest-path', 'tools/site-tools/Cargo.toml',
  '--bin', 'site-preview', '--', 'dist',
], { cwd: projectRoot, stdio: 'inherit' });
if (result.error) throw result.error;
process.exitCode = result.status ?? (result.signal === 'SIGINT' ? 130 : 1);
