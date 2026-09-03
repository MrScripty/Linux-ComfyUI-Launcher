import { createLinuxPlatformService } from './platform-linux.mjs';
import { createMacOSPlatformService } from './platform-macos.mjs';
import { createWindowsPlatformService } from './platform-windows.mjs';
import { EXIT_CODES } from './contract.mjs';
import { LauncherError } from './errors.mjs';

export function createPlatformService(platform = process.platform) {
  switch (platform) {
    case 'linux':
      return createLinuxPlatformService();
    case 'win32':
      return createWindowsPlatformService();
    case 'darwin':
      return createMacOSPlatformService();
    default:
      throw new LauncherError(`unsupported platform: ${platform}`, {
        exitCode: EXIT_CODES.UNSUPPORTED_PLATFORM,
      });
  }
}
