import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  testMatch: 'platform.spec.mjs',
  grep: /home to chapter|mobile drawer|reduced motion skips|reading indicator|interactive Vec|iterator laziness/,
  workers: 2,
  use: { trace:'retain-on-failure', colorScheme:'light' },
  projects: [
    { name:'firefox', use:{ ...devices['Desktop Firefox'] } },
    { name:'webkit', use:{ ...devices['Desktop Safari'] } },
    { name:'android-emulated', grep:/home to chapter|reduced motion skips|interactive Vec|iterator laziness/, use:{ ...devices['Pixel 7'] } },
  ],
});
