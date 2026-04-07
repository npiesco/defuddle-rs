const { test, expect } = require('@playwright/test');

test('parses HTML through the wasm package in a real browser', async ({ page }) => {
  await page.goto('/tests/wasm-smoke/', { waitUntil: 'networkidle' });
  await page.waitForFunction(() => window.__DEFUDDLE_WASM_RESULT__ || window.__DEFUDDLE_WASM_ERROR__);

  const payload = await page.evaluate(() => ({
    result: window.__DEFUDDLE_WASM_RESULT__ ?? null,
    error: window.__DEFUDDLE_WASM_ERROR__ ?? null,
  }));

  expect(payload.error).toBeNull();
  expect(payload.result).not.toBeNull();
  expect(payload.result.title).toBe('Test Article');
  expect(payload.result.author).toBe('Jane Doe');
  expect(payload.result.description).toBe('A test article for smoke testing');
  expect(payload.result.content_markdown).toBeTruthy();
  expect(payload.result.content_markdown).toContain('main content');
  expect(payload.result.word_count).toBeGreaterThan(0);
});
