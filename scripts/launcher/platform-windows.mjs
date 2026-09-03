import path from 'node:path';
import { spawn } from 'node:child_process';

export function createWindowsPlatformService({ spawnCommand = spawn } = {}) {
  return Object.freeze({
    id: 'win32',
    processTree: createWindowsProcessTree(spawnCommand),
    corepackCommand: 'corepack.cmd',
    cargoCommand: 'cargo.exe',
    pythonCommand: 'py.exe',
    pythonModuleArgs(moduleName, args = []) {
      return ['-3', '-m', moduleName, ...args];
    },
    debugBackendBinary(context) {
      return path.join(context.rustTargetDir, 'debug', `${context.appBin}.exe`);
    },
    releaseBackendBinary(context) {
      return path.join(context.rustTargetDir, 'release', `${context.appBin}.exe`);
    },
  });
}

function createWindowsProcessTree(spawnCommand) {
  return Object.freeze({
    spawnOptions: Object.freeze({ detached: false, windowsHide: false }),
    async terminate(child, { force, deadlineMs = 2_000 }) {
      if (!child.pid) {
        return;
      }

      if (!force) {
        const error = new Error('graceful process-tree termination is unavailable on Windows');
        error.code = 'WINDOWS_GRACEFUL_TERMINATION_UNAVAILABLE';
        throw error;
      }

      await runTaskkill(
        ['/pid', String(child.pid), '/t', '/f'],
        deadlineMs,
        spawnCommand
      );
    },
  });
}

async function runTaskkill(args, deadlineMs, spawnCommand) {
  await new Promise((resolve, reject) => {
    const boundedDeadlineMs = deadlineMs;
    const terminateAfterMs = Math.max(25, Math.floor(boundedDeadlineMs / 2));
    const closeAfterMs = Math.max(25, boundedDeadlineMs - terminateAfterMs - 25);
    const taskkill = spawnCommand('taskkill.exe', args, {
      shell: false,
      stdio: 'ignore',
      windowsHide: true,
    });
    let settled = false;
    let helperTimedOut = false;
    let exitOutcome;
    let terminateTimer;
    let closeTimer;

    const finish = (callback, value) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(terminateTimer);
      clearTimeout(closeTimer);
      callback(value);
    };

    taskkill.once('error', (error) => finish(reject, error));
    taskkill.once('spawn', () => {});
    taskkill.once('exit', (code, signal) => {
      exitOutcome = { code, signal };
    });
    taskkill.once('close', (code, signal) => {
      const observedCode = exitOutcome?.code ?? code;
      const observedSignal = exitOutcome?.signal ?? signal;

      if (helperTimedOut) {
        finish(reject, createTaskkillError('TASKKILL_HELPER_TIMEOUT'));
        return;
      }

      if (observedCode === 0 && !observedSignal) {
        finish(resolve);
        return;
      }

      const outcome = observedSignal ?? observedCode;
      finish(reject, createTaskkillError(`TASKKILL_EXIT_${outcome}`));
    });

    terminateTimer = setTimeout(() => {
      helperTimedOut = true;
      try {
        taskkill.kill('SIGKILL');
      } catch (error) {
        finish(reject, error);
        return;
      }
      closeTimer = setTimeout(() => {
        try {
          taskkill.kill('SIGKILL');
        } catch (error) {
          finish(reject, error);
          return;
        }
        finish(reject, createTaskkillError('TASKKILL_HELPER_CLOSE_TIMEOUT'));
      }, closeAfterMs);
    }, terminateAfterMs);
  });
}

function createTaskkillError(code) {
  const error = new Error(code);
  error.code = code;
  return error;
}
