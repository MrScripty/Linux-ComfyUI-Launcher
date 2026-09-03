import assert from 'node:assert/strict';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import {
  launcherRootOverrideConfigPath,
  resolveLauncherRoot,
} from '../dist/launcher-root.js';
import {
  LauncherRootRecoveryRequiredError,
  observeBackendInitialization,
  projectBackendInitializationFailure,
} from '../dist/startup-task.js';

function createLauncherRoot(root) {
  mkdirSync(join(root, 'shared-resources', 'models'), { recursive: true });
}

test('backend initialization rejection is observed while window creation is delayed', async () => {
  const recoveryFailure = new LauncherRootRecoveryRequiredError({
    status: 'recovery-required',
    code: 'launcher_root_invalid',
    authoritySource: 'persisted',
    persistedState: 'invalid',
    message: 'stable recovery required',
  });
  const unhandledRejections = [];
  const onUnhandledRejection = (reason) => {
    unhandledRejections.push(reason);
  };
  process.on('unhandledRejection', onUnhandledRejection);

  try {
    const initialization = observeBackendInitialization(Promise.reject(recoveryFailure));

    // Model createWindow remaining pending beyond the turn in which initialization rejects.
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(unhandledRejections, []);

    const outcome = await initialization;
    assert.equal(outcome.status, 'rejected');
    assert.equal(outcome.error, recoveryFailure);
    assert.deepEqual(
      projectBackendInitializationFailure('Failed to initialize backend bridge', outcome.error),
      {
        message:
          'Failed to initialize backend bridge: launcher_root_invalid (persisted): stable recovery required',
      }
    );

    // The closed observer preserves one typed terminal failure without raw logging.
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(unhandledRejections, []);
  } finally {
    process.off('unhandledRejection', onUnhandledRejection);
  }
});

test('persisted authority distinguishes absence and validity and blocks invalid discovery', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const discoveryRoot = join(fixtureRoot, 'discovery-root');
    const persistedRoot = join(fixtureRoot, 'persisted-root');
    const userDataPath = join(fixtureRoot, 'user-data');
    createLauncherRoot(discoveryRoot);
    createLauncherRoot(persistedRoot);

    const options = {
      argv: [],
      devRoot: join(fixtureRoot, 'development-default'),
      env: {},
      execPath: join(discoveryRoot, 'bin', 'pumas-library'),
      isPackaged: true,
      userDataPath,
    };

    mkdirSync(userDataPath, { recursive: true });
    writeFileSync(launcherRootOverrideConfigPath(userDataPath), '{\n', 'utf8');
    assert.deepEqual(resolveLauncherRoot(options), {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'persisted',
      persistedState: 'invalid',
      message: 'The saved launcher root is invalid; select an existing Pumas library to recover.',
    });

    unlinkSync(launcherRootOverrideConfigPath(userDataPath));
    assert.deepEqual(resolveLauncherRoot(options), {
      status: 'resolved',
      launcherRoot: discoveryRoot,
      source: 'discovery',
      persistedState: 'absent',
    });

    writeFileSync(
      launcherRootOverrideConfigPath(userDataPath),
      `${JSON.stringify({ launcherRoot: persistedRoot })}\n`,
      'utf8'
    );
    assert.deepEqual(resolveLauncherRoot(options), {
      status: 'resolved',
      launcherRoot: persistedRoot,
      source: 'persisted',
      persistedState: 'valid',
    });

  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('persisted metadata is non-authoritative and the JSON value must be a record', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const persistedRoot = join(fixtureRoot, 'persisted-root');
    const userDataPath = join(fixtureRoot, 'user-data');
    createLauncherRoot(persistedRoot);
    mkdirSync(userDataPath, { recursive: true });

    const options = {
      argv: [],
      devRoot: join(fixtureRoot, 'development-default'),
      env: {},
      execPath: join(fixtureRoot, 'pumas-library'),
      isPackaged: true,
      userDataPath,
    };

    writeFileSync(
      launcherRootOverrideConfigPath(userDataPath),
      `${JSON.stringify({
        launcherRoot: persistedRoot,
        selectedPath: 17,
        updatedAt: null,
        futureMetadata: { ignored: true },
      })}\n`,
      'utf8'
    );
    assert.deepEqual(resolveLauncherRoot(options), {
      status: 'resolved',
      launcherRoot: persistedRoot,
      source: 'persisted',
      persistedState: 'valid',
    });

    writeFileSync(
      launcherRootOverrideConfigPath(userDataPath),
      `${JSON.stringify([{ launcherRoot: persistedRoot }])}\n`,
      'utf8'
    );
    assert.deepEqual(resolveLauncherRoot(options), {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'persisted',
      persistedState: 'invalid',
      message: 'The saved launcher root is invalid; select an existing Pumas library to recover.',
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('unavailable persisted authority cannot fall through to discovery', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const discoveryRoot = join(fixtureRoot, 'discovery-root');
    const unavailableUserDataPath = join(fixtureRoot, 'user-data-is-a-file');
    createLauncherRoot(discoveryRoot);
    writeFileSync(unavailableUserDataPath, 'not a directory', 'utf8');

    assert.deepEqual(resolveLauncherRoot({
      argv: [],
      devRoot: join(fixtureRoot, 'development-default'),
      env: {},
      execPath: join(discoveryRoot, 'bin', 'pumas-library'),
      isPackaged: true,
      userDataPath: unavailableUserDataPath,
    }), {
      status: 'recovery-required',
      code: 'launcher_root_unavailable',
      authoritySource: 'persisted',
      persistedState: 'unavailable',
      message: 'The saved launcher root is unavailable; restore access or select a Pumas library to recover.',
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('invalid launcher-root argument is rejected before persisted state or discovery', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const discoveryRoot = join(fixtureRoot, 'discovery-root');
    const invalidArgumentRoot = join(fixtureRoot, 'missing-argument-root');
    createLauncherRoot(discoveryRoot);

    assert.deepEqual(resolveLauncherRoot({
      argv: ['pumas-library', '--launcher-root', invalidArgumentRoot],
      devRoot: join(fixtureRoot, 'development-default'),
      env: {},
      execPath: join(discoveryRoot, 'bin', 'pumas-library'),
      isPackaged: true,
      userDataPath: join(fixtureRoot, 'user-data'),
    }), {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'argument',
      persistedState: 'not-consulted',
      message: 'The selected launcher root is invalid; select an existing Pumas library to recover.',
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('invalid launcher-root environment override is rejected before argument and discovery', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const discoveryRoot = join(fixtureRoot, 'discovery-root');
    const validArgumentRoot = join(fixtureRoot, 'argument-root');
    const invalidEnvironmentRoot = join(fixtureRoot, 'missing-environment-root');
    createLauncherRoot(discoveryRoot);
    createLauncherRoot(validArgumentRoot);

    assert.deepEqual(resolveLauncherRoot({
      argv: ['pumas-library', `--launcher-root=${validArgumentRoot}`],
      devRoot: join(fixtureRoot, 'development-default'),
      env: { PUMAS_LAUNCHER_ROOT: invalidEnvironmentRoot },
      execPath: join(discoveryRoot, 'bin', 'pumas-library'),
      isPackaged: true,
      userDataPath: join(fixtureRoot, 'user-data'),
    }), {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'environment',
      persistedState: 'not-consulted',
      message: 'The selected launcher root is invalid; select an existing Pumas library to recover.',
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('valid explicit roots are normalized and skip persisted authority', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const argumentRoot = join(fixtureRoot, 'argument-root');
    const environmentRoot = join(fixtureRoot, 'environment-root');
    createLauncherRoot(argumentRoot);
    createLauncherRoot(environmentRoot);

    const baseOptions = {
      devRoot: join(fixtureRoot, 'development-default'),
      execPath: join(fixtureRoot, 'pumas-library'),
      isPackaged: true,
      userDataPath: join(fixtureRoot, 'user-data'),
    };

    assert.deepEqual(resolveLauncherRoot({
      ...baseOptions,
      argv: [
        'pumas-library',
        '--launcher-root',
        join(argumentRoot, 'shared-resources', 'models'),
      ],
      env: {},
    }), {
      status: 'resolved',
      launcherRoot: argumentRoot,
      source: 'argument',
      persistedState: 'not-consulted',
    });

    assert.deepEqual(resolveLauncherRoot({
      ...baseOptions,
      argv: ['pumas-library', `--launcher-root=${argumentRoot}`],
      env: { PUMAS_LAUNCHER_ROOT: join(environmentRoot, 'shared-resources') },
    }), {
      status: 'resolved',
      launcherRoot: environmentRoot,
      source: 'environment',
      persistedState: 'not-consulted',
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('launcher-root markers must be directories', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const invalidRoot = join(fixtureRoot, 'file-marker-root');
    mkdirSync(join(invalidRoot, 'shared-resources'), { recursive: true });
    writeFileSync(join(invalidRoot, 'shared-resources', 'models'), 'not a directory', 'utf8');

    assert.deepEqual(resolveLauncherRoot({
      argv: ['pumas-library', '--launcher-root', invalidRoot],
      devRoot: join(fixtureRoot, 'development-default'),
      env: {},
      execPath: join(fixtureRoot, 'pumas-library'),
      isPackaged: true,
      userDataPath: join(fixtureRoot, 'user-data'),
    }), {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'argument',
      persistedState: 'not-consulted',
      message: 'The selected launcher root is invalid; select an existing Pumas library to recover.',
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('authoritative root validation reports deterministic I/O failure as unavailable', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const launcherRoot = join(fixtureRoot, 'launcher-root');
    const userDataPath = join(fixtureRoot, 'user-data');
    createLauncherRoot(launcherRoot);
    mkdirSync(userDataPath, { recursive: true });
    writeFileSync(
      launcherRootOverrideConfigPath(userDataPath),
      `${JSON.stringify({ launcherRoot })}\n`,
      'utf8'
    );

    const unavailableFileSystem = {
      readFileSync,
      statSync() {
        const error = new Error('injected launcher-root validation failure');
        error.code = 'EIO';
        throw error;
      },
    };
    const baseOptions = {
      devRoot: join(fixtureRoot, 'development-default'),
      execPath: join(fixtureRoot, 'pumas-library'),
      isPackaged: true,
      userDataPath,
    };

    for (const authority of [
      {
        authoritySource: 'argument',
        argv: ['pumas-library', '--launcher-root', launcherRoot],
        env: {},
        persistedState: 'not-consulted',
      },
      {
        authoritySource: 'environment',
        argv: [],
        env: { PUMAS_LAUNCHER_ROOT: launcherRoot },
        persistedState: 'not-consulted',
      },
      {
        authoritySource: 'persisted',
        argv: [],
        env: {},
        persistedState: 'unavailable',
      },
    ]) {
      assert.deepEqual(resolveLauncherRoot({
        ...baseOptions,
        argv: authority.argv,
        env: authority.env,
      }, unavailableFileSystem), {
        status: 'recovery-required',
        code: 'launcher_root_unavailable',
        authoritySource: authority.authoritySource,
        persistedState: authority.persistedState,
        message: authority.authoritySource === 'persisted'
          ? 'The saved launcher root is unavailable; restore access or select a Pumas library to recover.'
          : 'The selected launcher root is unavailable; restore access or select a Pumas library to recover.',
      });
    }
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('malformed explicit and persisted path values are invalid, not unavailable', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const userDataPath = join(fixtureRoot, 'user-data');
    mkdirSync(userDataPath, { recursive: true });
    const baseOptions = {
      devRoot: join(fixtureRoot, 'development-default'),
      execPath: join(fixtureRoot, 'pumas-library'),
      isPackaged: true,
      userDataPath,
    };

    assert.deepEqual(resolveLauncherRoot({
      ...baseOptions,
      argv: ['pumas-library', '--launcher-root', 'invalid\0argument'],
      env: {},
    }), {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'argument',
      persistedState: 'not-consulted',
      message: 'The selected launcher root is invalid; select an existing Pumas library to recover.',
    });

    writeFileSync(
      launcherRootOverrideConfigPath(userDataPath),
      `${JSON.stringify({ launcherRoot: 'invalid\0persisted' })}\n`,
      'utf8'
    );
    assert.deepEqual(resolveLauncherRoot({
      ...baseOptions,
      argv: [],
      env: {},
    }), {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'persisted',
      persistedState: 'invalid',
      message: 'The saved launcher root is invalid; select an existing Pumas library to recover.',
    });
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('authoritative arbitrary descendants cannot normalize to a launcher-root ancestor', () => {
  const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

  try {
    const launcherRoot = join(fixtureRoot, 'launcher-root');
    const invalidDescendant = join(launcherRoot, 'not-an-authority-selection');
    const userDataPath = join(fixtureRoot, 'user-data');
    createLauncherRoot(launcherRoot);
    mkdirSync(userDataPath, { recursive: true });

    const baseOptions = {
      devRoot: join(fixtureRoot, 'development-default'),
      execPath: join(launcherRoot, 'bin', 'pumas-library'),
      isPackaged: true,
      userDataPath,
    };
    const cases = [
      {
        authoritySource: 'argument',
        argv: ['pumas-library', '--launcher-root', invalidDescendant],
        env: {},
        persistedState: 'not-consulted',
        message: 'The selected launcher root is invalid; select an existing Pumas library to recover.',
      },
      {
        authoritySource: 'environment',
        argv: [],
        env: { PUMAS_LAUNCHER_ROOT: invalidDescendant },
        persistedState: 'not-consulted',
        message: 'The selected launcher root is invalid; select an existing Pumas library to recover.',
      },
      {
        authoritySource: 'persisted',
        argv: [],
        env: {},
        persistedState: 'invalid',
        message: 'The saved launcher root is invalid; select an existing Pumas library to recover.',
      },
    ];

    for (const authority of cases) {
      if (authority.authoritySource === 'persisted') {
        writeFileSync(
          launcherRootOverrideConfigPath(userDataPath),
          `${JSON.stringify({ launcherRoot: invalidDescendant })}\n`,
          'utf8'
        );
      }

      assert.deepEqual(resolveLauncherRoot({
        ...baseOptions,
        argv: authority.argv,
        env: authority.env,
      }), {
        status: 'recovery-required',
        code: 'launcher_root_invalid',
        authoritySource: authority.authoritySource,
        persistedState: authority.persistedState,
        message: authority.message,
      });
    }
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

for (const explicitCase of [
  {
    name: 'missing argument value',
    argv: ['pumas-library', '--launcher-root'],
    env: {},
    authoritySource: 'argument',
  },
  {
    name: 'argument value replaced by another flag',
    argv: ['pumas-library', '--launcher-root', '--debug'],
    env: {},
    authoritySource: 'argument',
  },
  {
    name: 'blank inline argument value',
    argv: ['pumas-library', '--launcher-root=   '],
    env: {},
    authoritySource: 'argument',
  },
  {
    name: 'blank environment value',
    argv: [],
    env: { PUMAS_LAUNCHER_ROOT: '   ' },
    authoritySource: 'environment',
  },
]) {
  test(`present ${explicitCase.name} is invalid instead of absent`, () => {
    const fixtureRoot = mkdtempSync(join(tmpdir(), 'pumas-launcher-root-'));

    try {
      const discoveryRoot = join(fixtureRoot, 'discovery-root');
      createLauncherRoot(discoveryRoot);

      assert.deepEqual(resolveLauncherRoot({
        argv: explicitCase.argv,
        devRoot: join(fixtureRoot, 'development-default'),
        env: explicitCase.env,
        execPath: join(discoveryRoot, 'bin', 'pumas-library'),
        isPackaged: true,
        userDataPath: join(fixtureRoot, 'user-data'),
      }), {
        status: 'recovery-required',
        code: 'launcher_root_invalid',
        authoritySource: explicitCase.authoritySource,
        persistedState: 'not-consulted',
        message: 'The selected launcher root is invalid; select an existing Pumas library to recover.',
      });
    } finally {
      rmSync(fixtureRoot, { recursive: true, force: true });
    }
  });
}
