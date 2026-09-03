import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const FRONTEND_HTML = readFileSync(
  new URL('../../frontend/index.html', import.meta.url),
  'utf8'
);

test('frontend declares an Electron-safe content security policy', () => {
  const contentSecurityPolicy = FRONTEND_HTML.match(
    /<meta\s+http-equiv="Content-Security-Policy"\s+content="([^"]+)"\s*\/?>/
  )?.[1];

  assert.ok(contentSecurityPolicy, 'frontend/index.html must declare a Content Security Policy');
  assert.match(contentSecurityPolicy, /(?:^|;)\s*default-src\s+'self'/);
  assert.match(contentSecurityPolicy, /(?:^|;)\s*script-src\s+'self'/);
  assert.doesNotMatch(contentSecurityPolicy, /'unsafe-eval'/);
});
