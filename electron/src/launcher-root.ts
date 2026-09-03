import * as fs from 'fs';
import * as path from 'path';

export interface LauncherRootResolutionOptions {
  appImagePath?: string;
  argv?: string[];
  devRoot: string;
  env?: NodeJS.ProcessEnv;
  execPath: string;
  isPackaged: boolean;
  userDataPath: string;
}

export interface LauncherRootFileSystem {
  readFileSync(filePath: string, encoding: 'utf8'): string;
  statSync(filePath: string): { isDirectory(): boolean };
}

export type PersistedLauncherRootState =
  | 'not-consulted'
  | 'absent'
  | 'valid'
  | 'invalid'
  | 'unavailable';

export type LauncherRootSource =
  | 'environment'
  | 'argument'
  | 'persisted'
  | 'appimage-portable'
  | 'discovery'
  | 'appimage-default'
  | 'packaged-default'
  | 'development-default';

export type LauncherRootResolution =
  | {
      status: 'resolved';
      launcherRoot: string;
      source: 'environment' | 'argument';
      persistedState: 'not-consulted';
    }
  | {
      status: 'resolved';
      launcherRoot: string;
      source: 'persisted';
      persistedState: 'valid';
    }
  | {
      status: 'resolved';
      launcherRoot: string;
      source: Exclude<LauncherRootSource, 'environment' | 'argument' | 'persisted'>;
      persistedState: 'absent';
    }
  | {
      status: 'recovery-required';
      code: 'launcher_root_invalid' | 'launcher_root_unavailable';
      authoritySource: 'environment' | 'argument';
      persistedState: 'not-consulted';
      message: string;
    }
  | {
      status: 'recovery-required';
      code: 'launcher_root_invalid';
      authoritySource: 'persisted';
      persistedState: 'invalid';
      message: string;
    }
  | {
      status: 'recovery-required';
      code: 'launcher_root_unavailable';
      authoritySource: 'persisted';
      persistedState: 'unavailable';
      message: string;
    };

interface PersistedLauncherRootConfig {
  launcherRoot: string;
  selectedPath?: string;
  updatedAt: string;
}

const LAUNCHER_ROOT_OVERRIDE_FILENAME = 'launcher-root.json';
const INVALID_PERSISTED_AUTHORITY_MESSAGE =
  'The saved launcher root is invalid; select an existing Pumas library to recover.';
const UNAVAILABLE_PERSISTED_AUTHORITY_MESSAGE =
  'The saved launcher root is unavailable; restore access or select a Pumas library to recover.';
const INVALID_EXPLICIT_AUTHORITY_MESSAGE =
  'The selected launcher root is invalid; select an existing Pumas library to recover.';
const UNAVAILABLE_EXPLICIT_AUTHORITY_MESSAGE =
  'The selected launcher root is unavailable; restore access or select a Pumas library to recover.';
const NODE_LAUNCHER_ROOT_FILE_SYSTEM: LauncherRootFileSystem = {
  readFileSync: (filePath, encoding) => fs.readFileSync(filePath, encoding),
  statSync: (filePath) => fs.statSync(filePath),
};
const INVALID_PATH_ERROR_CODES = new Set([
  'ENOENT',
  'ENOTDIR',
  'EINVAL',
  'ENAMETOOLONG',
  'ERR_INVALID_ARG_VALUE',
]);

type ExplicitLauncherRoot =
  | { state: 'absent' }
  | { state: 'invalid'; source: 'environment' | 'argument' }
  | {
      state: 'selected';
      launcherRoot: string;
      source: 'environment' | 'argument';
    };

type PersistedLauncherRootAuthority =
  | { state: 'absent' }
  | { state: 'valid'; launcherRoot: string }
  | { state: 'invalid' }
  | { state: 'unavailable' };

type LauncherRootValidation =
  | { state: 'valid'; launcherRoot: string }
  | { state: 'invalid' }
  | { state: 'unavailable' };

type DirectoryInspection = 'directory' | 'invalid' | 'unavailable';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function resolveLauncherRoot(
  options: LauncherRootResolutionOptions,
  fileSystem: LauncherRootFileSystem = NODE_LAUNCHER_ROOT_FILE_SYSTEM
): LauncherRootResolution {
  const override = resolveLauncherRootOverride(
    options.argv ?? process.argv,
    options.env ?? process.env
  );
  if (override.state === 'invalid') {
    return {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: override.source,
      persistedState: 'not-consulted',
      message: INVALID_EXPLICIT_AUTHORITY_MESSAGE,
    };
  }
  if (override.state === 'selected') {
    const validation = validateLauncherRootSelection(override.launcherRoot, fileSystem);
    if (validation.state !== 'valid') {
      return {
        status: 'recovery-required',
        code: validation.state === 'unavailable'
          ? 'launcher_root_unavailable'
          : 'launcher_root_invalid',
        authoritySource: override.source,
        persistedState: 'not-consulted',
        message: validation.state === 'unavailable'
          ? UNAVAILABLE_EXPLICIT_AUTHORITY_MESSAGE
          : INVALID_EXPLICIT_AUTHORITY_MESSAGE,
      };
    }
    return {
      status: 'resolved',
      launcherRoot: validation.launcherRoot,
      source: override.source,
      persistedState: 'not-consulted',
    };
  }

  const persistedAuthority = readPersistedLauncherRootAuthority(
    options.userDataPath,
    fileSystem
  );
  if (persistedAuthority.state === 'invalid') {
    return {
      status: 'recovery-required',
      code: 'launcher_root_invalid',
      authoritySource: 'persisted',
      persistedState: 'invalid',
      message: INVALID_PERSISTED_AUTHORITY_MESSAGE,
    };
  }
  if (persistedAuthority.state === 'unavailable') {
    return {
      status: 'recovery-required',
      code: 'launcher_root_unavailable',
      authoritySource: 'persisted',
      persistedState: 'unavailable',
      message: UNAVAILABLE_PERSISTED_AUTHORITY_MESSAGE,
    };
  }
  if (persistedAuthority.state === 'valid') {
    return {
      status: 'resolved',
      launcherRoot: persistedAuthority.launcherRoot,
      source: 'persisted',
      persistedState: 'valid',
    };
  }

  const appImagePortableRoot = options.appImagePath
    ? path.join(path.dirname(options.appImagePath), 'pumas-data')
    : undefined;

  if (appImagePortableRoot && isExistingLauncherRoot(appImagePortableRoot, fileSystem)) {
    return resolvedLauncherRoot(appImagePortableRoot, 'appimage-portable');
  }

  const candidateStarts = new Set<string>();
  if (options.appImagePath) {
    candidateStarts.add(path.dirname(options.appImagePath));
  }
  if (options.isPackaged) {
    candidateStarts.add(path.dirname(options.execPath));
  }

  for (const startDir of candidateStarts) {
    const existingRoot = findLauncherRootFrom(startDir, fileSystem);
    if (existingRoot) {
      return resolvedLauncherRoot(existingRoot, 'discovery');
    }
  }

  if (appImagePortableRoot) {
    return resolvedLauncherRoot(appImagePortableRoot, 'appimage-default');
  }

  if (options.isPackaged) {
    return resolvedLauncherRoot(options.userDataPath, 'packaged-default');
  }

  return resolvedLauncherRoot(options.devRoot, 'development-default');
}

function resolvedLauncherRoot(
  launcherRoot: string,
  source: Exclude<LauncherRootSource, 'environment' | 'argument' | 'persisted'>
): LauncherRootResolution {
  return {
    status: 'resolved',
    launcherRoot,
    source,
    persistedState: 'absent',
  };
}

export function persistLauncherRootOverride(
  userDataPath: string,
  selectedPath: string
): PersistedLauncherRootConfig {
  const validation = validateLauncherRootSelection(
    selectedPath,
    NODE_LAUNCHER_ROOT_FILE_SYSTEM
  );
  if (validation.state !== 'valid') {
    throw new Error(
      'Selected path must be a launcher root, shared-resources directory, or shared-resources/models directory.'
    );
  }
  const launcherRoot = validation.launcherRoot;

  fs.mkdirSync(userDataPath, { recursive: true });

  const config: PersistedLauncherRootConfig = {
    launcherRoot,
    selectedPath: path.resolve(selectedPath),
    updatedAt: new Date().toISOString(),
  };

  fs.writeFileSync(
    launcherRootOverrideConfigPath(userDataPath),
    `${JSON.stringify(config, null, 2)}\n`,
    'utf8'
  );

  return config;
}

export function launcherRootOverrideConfigPath(userDataPath: string): string {
  return path.join(userDataPath, LAUNCHER_ROOT_OVERRIDE_FILENAME);
}

function resolveLauncherRootOverride(
  argv: string[],
  env: NodeJS.ProcessEnv
): ExplicitLauncherRoot {
  const rawEnvironmentOverride = env.PUMAS_LAUNCHER_ROOT;
  if (rawEnvironmentOverride !== undefined) {
    const envOverride = rawEnvironmentOverride.trim();
    if (!envOverride) {
      return { state: 'invalid', source: 'environment' };
    }
    return {
      state: 'selected',
      launcherRoot: path.resolve(envOverride),
      source: 'environment',
    };
  }

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];

    if (current === '--launcher-root') {
      const next = argv[index + 1]?.trim();
      if (!next || next.startsWith('--')) {
        return { state: 'invalid', source: 'argument' };
      }
      return {
        state: 'selected',
        launcherRoot: path.resolve(next),
        source: 'argument',
      };
    }

    if (current.startsWith('--launcher-root=')) {
      const value = current.slice('--launcher-root='.length).trim();
      if (!value) {
        return { state: 'invalid', source: 'argument' };
      }
      return {
        state: 'selected',
        launcherRoot: path.resolve(value),
        source: 'argument',
      };
    }
  }

  return { state: 'absent' };
}

function readPersistedLauncherRootAuthority(
  userDataPath: string,
  fileSystem: LauncherRootFileSystem
): PersistedLauncherRootAuthority {
  const configPath = launcherRootOverrideConfigPath(userDataPath);
  let serializedConfig: string;

  try {
    serializedConfig = fileSystem.readFileSync(configPath, 'utf8');
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return { state: 'absent' };
    }
    return { state: 'unavailable' };
  }

  try {
    const parsed: unknown = JSON.parse(serializedConfig);
    if (!isRecord(parsed)) {
      return { state: 'invalid' };
    }

    const configuredRoot = typeof parsed['launcherRoot'] === 'string'
      ? parsed['launcherRoot']
      : '';
    if (!configuredRoot) {
      return { state: 'invalid' };
    }

    return validateLauncherRootSelection(configuredRoot, fileSystem);
  } catch {
    return { state: 'invalid' };
  }
}

function validateLauncherRootSelection(
  selectedPath: string,
  fileSystem: LauncherRootFileSystem
): LauncherRootValidation {
  const resolved = path.resolve(selectedPath);
  const candidates = new Set<string>([resolved]);

  if (path.basename(resolved) === 'models' && path.basename(path.dirname(resolved)) === 'shared-resources') {
    candidates.add(path.dirname(path.dirname(resolved)));
  }

  if (path.basename(resolved) === 'shared-resources') {
    candidates.add(path.dirname(resolved));
  }

  for (const candidate of candidates) {
    const validation = validateLauncherRootLayout(candidate, fileSystem);
    if (validation === 'valid') {
      return { state: 'valid', launcherRoot: candidate };
    }
    if (validation === 'unavailable') {
      return { state: 'unavailable' };
    }
  }

  return { state: 'invalid' };
}

function findLauncherRootFrom(
  startDir: string,
  fileSystem: LauncherRootFileSystem
): string | null {
  let current = path.resolve(startDir);

  while (true) {
    const validation = validateLauncherRootLayout(current, fileSystem);
    if (validation === 'valid') {
      return current;
    }
    if (validation === 'unavailable') {
      return null;
    }

    const parent = path.dirname(current);
    if (parent === current) {
      return null;
    }
    current = parent;
  }
}

function isExistingLauncherRoot(
  candidate: string,
  fileSystem: LauncherRootFileSystem
): boolean {
  return validateLauncherRootLayout(candidate, fileSystem) === 'valid';
}

function validateLauncherRootLayout(
  candidate: string,
  fileSystem: LauncherRootFileSystem
): 'valid' | 'invalid' | 'unavailable' {
  const models = inspectDirectory(
    path.join(candidate, 'shared-resources', 'models'),
    fileSystem
  );
  const launcherData = inspectDirectory(path.join(candidate, 'launcher-data'), fileSystem);
  const sharedResources = inspectDirectory(
    path.join(candidate, 'shared-resources'),
    fileSystem
  );

  if (
    models === 'directory' ||
    (launcherData === 'directory' && sharedResources === 'directory')
  ) {
    return 'valid';
  }
  if (
    models === 'unavailable' ||
    launcherData === 'unavailable' ||
    sharedResources === 'unavailable'
  ) {
    return 'unavailable';
  }
  return 'invalid';
}

function inspectDirectory(
  candidate: string,
  fileSystem: LauncherRootFileSystem
): DirectoryInspection {
  try {
    return fileSystem.statSync(candidate).isDirectory() ? 'directory' : 'invalid';
  } catch (error) {
    const code = (error as NodeJS.ErrnoException).code;
    return code && INVALID_PATH_ERROR_CODES.has(code) ? 'invalid' : 'unavailable';
  }
}
