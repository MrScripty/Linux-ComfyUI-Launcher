export function createPosixProcessTree() {
  return Object.freeze({
    spawnOptions: Object.freeze({ detached: true }),
    terminate(child, { force }) {
      terminateProcessGroup(child, force ? 'SIGKILL' : 'SIGTERM');
    },
  });
}

function terminateProcessGroup(child, signal) {
  if (!child.pid) {
    return;
  }

  try {
    process.kill(-child.pid, signal);
  } catch (error) {
    if (error?.code === 'ESRCH') {
      return;
    }

    throw error;
  }
}
