import type { LauncherRootResolution } from './launcher-root';

type LauncherRootRecoveryRequired = Extract<
  LauncherRootResolution,
  { status: 'recovery-required' }
>;

export type BackendInitializationOutcome =
  | { status: 'fulfilled' }
  | { status: 'rejected'; error: unknown };

export type BackendInitializationFailureDiagnostic =
  | { message: string }
  | { message: string; error: unknown };

export class LauncherRootRecoveryRequiredError extends Error {
  readonly code: LauncherRootRecoveryRequired['code'];
  readonly authoritySource: LauncherRootRecoveryRequired['authoritySource'];

  constructor(resolution: LauncherRootRecoveryRequired) {
    super(resolution.message);
    this.name = 'LauncherRootRecoveryRequiredError';
    this.code = resolution.code;
    this.authoritySource = resolution.authoritySource;
  }
}

/**
 * Observe backend initialization immediately and preserve its one terminal outcome.
 *
 * Window creation may remain pending after backend initialization fails. Converting
 * the task to this closed outcome at creation time prevents that failure from
 * reaching the process-level unhandled-rejection diagnostic before the window is
 * ready to consume it.
 */
export function observeBackendInitialization(
  initialization: Promise<void>
): Promise<BackendInitializationOutcome> {
  return initialization.then<BackendInitializationOutcome, BackendInitializationOutcome>(
    () => ({ status: 'fulfilled' }),
    (error: unknown) => ({ status: 'rejected', error })
  );
}

export function projectBackendInitializationFailure(
  message: string,
  error: unknown
): BackendInitializationFailureDiagnostic {
  if (error instanceof LauncherRootRecoveryRequiredError) {
    return {
      message: `${message}: ${error.code} (${error.authoritySource}): ${error.message}`,
    };
  }

  return { message, error };
}
