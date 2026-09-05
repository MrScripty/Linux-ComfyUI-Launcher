import type { LauncherRootResolution } from './launcher-root';
import {
  projectLauncherRootStartupState,
  type LauncherRootStartupState,
} from './launcher-root-recovery';

type LauncherRootRecoveryRequired = Extract<
  LauncherRootResolution,
  { status: 'recovery-required' }
>;

export type BackendInitializationOutcome =
  | { status: 'fulfilled' }
  | { status: 'rejected'; error: unknown };

export type BackendInitializationMode = 'desktop' | 'release-smoke';

export type BackendInitializationDisposition =
  | { status: 'ready' }
  | {
      status: 'recovery-required';
      recoveryState: Extract<
        LauncherRootStartupState,
        { status: 'recovery-required' }
      >;
    }
  | { status: 'fatal'; error: unknown };

export type BackendInitializationFailureDiagnostic =
  | { message: string }
  | { message: string; error: unknown };

export class LauncherRootRecoveryRequiredError extends Error {
  readonly code: LauncherRootRecoveryRequired['code'];
  readonly authoritySource: LauncherRootRecoveryRequired['authoritySource'];
  readonly recoveryState: Extract<
    LauncherRootStartupState,
    { status: 'recovery-required' }
  >;

  constructor(resolution: LauncherRootRecoveryRequired) {
    super(resolution.message);
    this.name = 'LauncherRootRecoveryRequiredError';
    this.code = resolution.code;
    this.authoritySource = resolution.authoritySource;
    this.recoveryState = projectLauncherRootStartupState(resolution);
  }
}

export function isLauncherRootRecoveryRequiredError(
  error: unknown
): error is LauncherRootRecoveryRequiredError {
  return error instanceof LauncherRootRecoveryRequiredError;
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

export function classifyBackendInitializationOutcome(
  outcome: BackendInitializationOutcome,
  mode: BackendInitializationMode
): BackendInitializationDisposition {
  if (outcome.status === 'fulfilled') {
    return { status: 'ready' };
  }

  if (mode === 'desktop' && isLauncherRootRecoveryRequiredError(outcome.error)) {
    return {
      status: 'recovery-required',
      recoveryState: outcome.error.recoveryState,
    };
  }

  return { status: 'fatal', error: outcome.error };
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
