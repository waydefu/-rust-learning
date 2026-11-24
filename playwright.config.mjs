import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  workers: 2,
  use: {
    ...devices['Desktop Chrome'],
    viewport: { width: 1440, height: 1000 },
    launchOptions: process.env.BROWSER_EXECUTABLE ? { executablePath: process.env.BROWSER_EXECUTABLE } : {},
    colorScheme: 'light',
    trace: 'retain-on-failure',
  },
});
