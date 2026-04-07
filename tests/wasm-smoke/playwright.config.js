const { defineConfig } = require('@playwright/test');
const path = require('path');

module.exports = defineConfig({
  testDir: '.',
  testMatch: ['wasm.spec.js'],
  reporter: 'list',
  timeout: 120000,
  use: {
    baseURL: 'http://127.0.0.1:4173',
    browserName: 'chromium',
    headless: true,
  },
  webServer: {
    command: 'node static-server.cjs 4173 ../..',
    cwd: path.resolve(__dirname),
    url: 'http://127.0.0.1:4173/tests/wasm-smoke/',
    reuseExistingServer: true,
    timeout: 120000,
  },
});
