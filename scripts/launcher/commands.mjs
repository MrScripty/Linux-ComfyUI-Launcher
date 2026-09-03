import { spawn, spawnSync } from 'node:child_process';
import { LauncherError } from './errors.mjs';
import { EXIT_CODES } from './contract.mjs';

export function commandExists(command, args = ['--version']) {
  const result = spawnSync(command, args, { stdio: 'ignore' });
  return result.status === 0;
}

export async function runCommand(command, args, options = {}) {
  const env = { ...process.env, ...options.env };

  await new Promise((resolve, reject) => {
    let child;
    try {
      child = spawn(command, args, {
        cwd: options.cwd,
        env,
        stdio: 'inherit',
        shell: false,
      });
    } catch (error) {
      reject(createSpawnError(command, error));
      return;
    }
    let settled = false;
    let spawned = false;
    let exitOutcome;

    const finish = (callback, value) => {
      if (settled) {
        return;
      }

      settled = true;
      callback(value);
    };

    child.once('error', (error) => {
      finish(reject, createSpawnError(command, error));
    });

    child.once('spawn', () => {
      spawned = true;
    });

    child.once('exit', (code, signal) => {
      exitOutcome = { code, signal };
    });

    child.once('close', (code, signal) => {
      const observedCode = exitOutcome?.code ?? code;
      const observedSignal = exitOutcome?.signal ?? signal;

      if (!spawned) {
        finish(
          reject,
          new LauncherError(`${describeCommand(command)} closed before spawn`, {
            exitCode: EXIT_CODES.OPERATION_FAILED,
          })
        );
        return;
      }

      if (observedSignal) {
        finish(
          reject,
          new LauncherError(
            `${describeCommand(command)} terminated by signal ${observedSignal}`,
            { exitCode: EXIT_CODES.OPERATION_FAILED }
          )
        );
        return;
      }

      if (observedCode !== 0) {
        finish(
          reject,
          new LauncherError(
            `${describeCommand(command)} exited with code ${observedCode}`,
            { exitCode: EXIT_CODES.OPERATION_FAILED }
          )
        );
        return;
      }

      finish(resolve);
    });
  });
}

export async function runBoundedCommand(command, args, options = {}) {
  const env = { ...process.env, ...options.env };
  const minUptimeMs = options.minUptimeMs ?? 0;
  const maxUptimeMs = options.maxUptimeMs ?? 30_000;
  const terminationGraceMs = options.terminationGraceMs ?? 2_000;
  const forceCloseMs = options.forceCloseMs ?? 2_000;
  const processTree = options.processTree;

  validateBoundedDurations({
    minUptimeMs,
    maxUptimeMs,
    terminationGraceMs,
    forceCloseMs,
  });

  if (!processTree?.spawnOptions || typeof processTree.terminate !== 'function') {
    throw new LauncherError('bounded command requires a platform process-tree adapter', {
      exitCode: EXIT_CODES.OPERATION_FAILED,
    });
  }

  await new Promise((resolve, reject) => {
    const startedAt = Date.now();
    let timedOut = false;
    let forceStarted = false;
    let forceWindowExpired = false;
    let settled = false;
    let childClosed = false;
    let pendingTerminations = 0;
    let exitOutcome;
    let terminationFailure;
    let maxTimer;
    let graceTimer;
    let forceTimer;

    let child;
    try {
      child = spawn(command, args, {
        cwd: options.cwd,
        env,
        ...processTree.spawnOptions,
        stdio: 'inherit',
        shell: false,
      });
    } catch (error) {
      reject(createSpawnError(command, error));
      return;
    }

    const clearTimers = () => {
      clearTimeout(maxTimer);
      clearTimeout(graceTimer);
      clearTimeout(forceTimer);
    };

    const finish = (callback, value) => {
      if (settled) {
        return;
      }

      settled = true;
      clearTimers();
      callback(value);
    };

    const fail = (message) => {
      finish(
        reject,
        new LauncherError(message, { exitCode: EXIT_CODES.OPERATION_FAILED })
      );
    };

    const requestTermination = (force) => {
      pendingTerminations += 1;
      Promise.resolve()
        .then(() => {
          if (!settled) {
            return processTree.terminate(child, {
              force,
              deadlineMs: force ? forceCloseMs : terminationGraceMs,
            });
          }
        })
        .catch((error) => {
          terminationFailure = describeTerminationFailure(error);
          if (!force) {
            beginForcedTermination();
          }
        })
        .finally(() => {
          pendingTerminations -= 1;
          finishTimedOutCommandWhenOwnedWorkStops();
        });
    };

    const finishTimedOutCommandWhenOwnedWorkStops = () => {
      if (settled || !timedOut || pendingTerminations !== 0) {
        return;
      }

      const detail = terminationFailure
        ? `; termination adapter reported ${terminationFailure}`
        : '';
      if (childClosed) {
        fail(`${describeCommand(command)} exceeded smoke window (${maxUptimeMs}ms)${detail}`);
        return;
      }

      if (forceWindowExpired) {
        fail(
          `${describeCommand(command)} exceeded smoke window (${maxUptimeMs}ms) and did not close within the forced termination window (${forceCloseMs}ms)${detail}`
        );
      }
    };

    const beginForcedTermination = () => {
      if (settled || forceStarted) {
        return;
      }

      forceStarted = true;
      clearTimeout(graceTimer);
      requestTermination(true);
      forceTimer = setTimeout(() => {
        forceWindowExpired = true;
        finishTimedOutCommandWhenOwnedWorkStops();
      }, forceCloseMs);
    };

    child.once('error', (error) => {
      fail(`failed to start ${describeCommand(command)}: ${describeProcessError(error)}`);
    });

    child.once('spawn', () => {
      maxTimer = setTimeout(() => {
        timedOut = true;
        requestTermination(false);
        graceTimer = setTimeout(beginForcedTermination, terminationGraceMs);
      }, maxUptimeMs);
    });

    child.once('exit', (code, signal) => {
      exitOutcome = { code, signal };
      if (!timedOut) {
        clearTimeout(maxTimer);
      }
    });

    child.once('close', (code, signal) => {
      childClosed = true;
      const elapsedMs = Date.now() - startedAt;
      const observedCode = exitOutcome?.code ?? code;
      const observedSignal = exitOutcome?.signal ?? signal;

      if (timedOut) {
        finishTimedOutCommandWhenOwnedWorkStops();
        return;
      }

      if (observedSignal) {
        fail(`${describeCommand(command)} terminated by signal ${observedSignal}`);
        return;
      }

      if (observedCode !== 0) {
        fail(`${describeCommand(command)} exited with code ${observedCode}`);
        return;
      }

      if (elapsedMs < minUptimeMs) {
        fail(
          `${describeCommand(command)} exited before the minimum smoke window (${elapsedMs}ms < ${minUptimeMs}ms)`
        );
        return;
      }

      finish(resolve);
    });
  });
}

function describeTerminationFailure(error) {
  if (error && typeof error === 'object' && 'code' in error) {
    return String(error.code);
  }

  return error instanceof Error ? error.name : 'unknown failure';
}

function validateBoundedDurations({
  minUptimeMs,
  maxUptimeMs,
  terminationGraceMs,
  forceCloseMs,
}) {
  const values = [minUptimeMs, maxUptimeMs, terminationGraceMs, forceCloseMs];
  if (!values.every(Number.isFinite)
    || minUptimeMs < 0
    || maxUptimeMs <= 0
    || terminationGraceMs < 0
    || forceCloseMs < 100
    || minUptimeMs > maxUptimeMs) {
    throw new LauncherError('invalid bounded command deadlines', {
      exitCode: EXIT_CODES.OPERATION_FAILED,
    });
  }
}

function describeCommand(command) {
  const segments = String(command).replaceAll('\\', '/').split('/');
  return segments.at(-1) || 'launcher child process';
}

function describeProcessError(error) {
  if (error && typeof error === 'object' && 'code' in error) {
    return String(error.code);
  }

  return error instanceof Error ? error.name : 'unknown failure';
}

function createSpawnError(command, error) {
  return new LauncherError(
    `failed to start ${describeCommand(command)}: ${describeProcessError(error)}`,
    { exitCode: EXIT_CODES.OPERATION_FAILED }
  );
}
