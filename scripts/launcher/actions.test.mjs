import assert from 'node:assert/strict';
import test from 'node:test';
import { areInferencePluginsEnabled, resolveReleaseSmokeScript } from './actions.mjs';

test('inference plugins are enabled by default and can be compiled out', () => {
  assert.equal(areInferencePluginsEnabled({}), true);
  assert.equal(areInferencePluginsEnabled({ PUMAS_INFERENCE_PLUGINS: 'true' }), true);
  assert.equal(areInferencePluginsEnabled({ PUMAS_INFERENCE_PLUGINS: 'false' }), false);
});

test('resolveReleaseSmokeScript selects the CI-safe Electron entrypoint on Linux CI', () => {
  const previousCi = process.env.CI;
  process.env.CI = 'true';

  try {
    assert.equal(resolveReleaseSmokeScript({ id: 'linux' }), 'run:launcher-release-ci-smoke');
  } finally {
    restoreCi(previousCi);
  }
});

test('resolveReleaseSmokeScript keeps the standard Electron entrypoint outside Linux CI', () => {
  const previousCi = process.env.CI;
  process.env.CI = 'false';

  try {
    assert.equal(resolveReleaseSmokeScript({ id: 'linux' }), 'run:launcher-release');
    assert.equal(resolveReleaseSmokeScript({ id: 'darwin' }), 'run:launcher-release');
    assert.equal(resolveReleaseSmokeScript({ id: 'win32' }), 'run:launcher-release');
  } finally {
    restoreCi(previousCi);
  }
});

function restoreCi(previousCi) {
  if (previousCi === undefined) {
    delete process.env.CI;
    return;
  }

  process.env.CI = previousCi;
}
