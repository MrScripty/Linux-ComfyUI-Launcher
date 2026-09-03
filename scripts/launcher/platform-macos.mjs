import path from 'node:path';
import { createPosixProcessTree } from './platform-posix-process.mjs';

export function createMacOSPlatformService() {
  return Object.freeze({
    id: 'darwin',
    processTree: createPosixProcessTree(),
    corepackCommand: 'corepack',
    cargoCommand: 'cargo',
    pythonCommand: 'python3',
    pythonModuleArgs(moduleName, args = []) {
      return ['-m', moduleName, ...args];
    },
    debugBackendBinary(context) {
      return path.join(context.rustTargetDir, 'debug', context.appBin);
    },
    releaseBackendBinary(context) {
      return path.join(context.rustTargetDir, 'release', context.appBin);
    },
  });
}
