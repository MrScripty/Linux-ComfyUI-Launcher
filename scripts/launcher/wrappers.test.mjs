import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const moduleDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(moduleDir, '..', '..');
const launcherSh = path.join(repoRoot, 'launcher.sh');
const launcherPs1 = path.join(repoRoot, 'launcher.ps1');

test('launcher.sh stays a thin wrapper over the shared core', () => {
  const contents = fs.readFileSync(launcherSh, 'utf8');

  assert.match(contents, /PUMAS_LAUNCHER_DISPLAY_NAME='\.\/*launcher\.sh'/);
  assert.match(contents, /exec node "\$LAUNCHER_CORE" "\$@"/);
  assert.doesNotMatch(contents, /RELEASE_(?:BACKEND|FRONTEND|ELECTRON)/);
  assert.doesNotMatch(contents, /exec "\$RELEASE_ELECTRON_BINARY"/);
  assert.doesNotMatch(contents, /cargo build/);
  assert.doesNotMatch(contents, /npm --workspace/);
  assert.match(contents, /node missing[\s\S]*exit 3/);
});

test('launcher.ps1 stays a thin wrapper over the shared core', () => {
  const contents = fs.readFileSync(launcherPs1, 'utf8');

  assert.match(contents, /\$env:PUMAS_LAUNCHER_DISPLAY_NAME = '\.\/launcher\.ps1'/);
  assert.match(contents, /& \$nodeCommand\.Source \$launcherCore @args/);
  assert.doesNotMatch(contents, /cargo build/);
  assert.doesNotMatch(contents, /npm --workspace/);
  assert.match(contents, /node missing[\s\S]*exit 3/);
});

test('native launcher wrapper preserves help and usage-error exit codes', () => {
  const wrapper = nativeWrapperInvocation();
  const help = spawnSync(wrapper.command, [...wrapper.args, '--help'], {
    cwd: repoRoot,
    encoding: 'utf8',
    shell: false,
  });

  assert.equal(help.error, undefined);
  assert.equal(help.status, 0, help.stderr);
  assert.match(help.stdout, /Pumas Library launcher/);

  const invalid = spawnSync(wrapper.command, [...wrapper.args, '--not-an-action'], {
    cwd: repoRoot,
    encoding: 'utf8',
    shell: false,
  });

  assert.equal(invalid.error, undefined);
  assert.equal(invalid.status, 2, invalid.stderr);
  assert.match(invalid.stderr, /unknown argument/);
});

function nativeWrapperInvocation() {
  if (process.platform === 'win32') {
    return {
      command: 'powershell.exe',
      args: [
        '-NoProfile',
        '-NonInteractive',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        launcherPs1,
      ],
    };
  }

  return { command: 'bash', args: [launcherSh] };
}
