export type LauncherRootCommittedPresentation =
  | 'ready'
  | 'recovery-required'
  | 'bridge-unavailable';

export type LauncherRootAuthoritativeStatus =
  | 'initializing'
  | 'ready'
  | 'recovery-required';

const PRESENTATION_MARKER_DARK = 37;
const PRESENTATION_MARKER_LIGHT = 211;
const PRESENTATION_MARKER_TOLERANCE = 8;

export const WINDOW_PRESENTATION_MARKER_CSS = `
html::after {
  content: '' !important;
  position: fixed !important;
  inset: 0 !important;
  z-index: 2147483647 !important;
  pointer-events: none !important;
  opacity: 1 !important;
  background:
    linear-gradient(rgb(37, 37, 37), rgb(37, 37, 37)) 0 0 / 33.34% 33.34% no-repeat,
    linear-gradient(rgb(37, 37, 37), rgb(37, 37, 37)) 100% 0 / 33.34% 33.34% no-repeat,
    linear-gradient(rgb(37, 37, 37), rgb(37, 37, 37)) 50% 50% / 33.34% 33.34% no-repeat,
    linear-gradient(rgb(37, 37, 37), rgb(37, 37, 37)) 0 100% / 33.34% 33.34% no-repeat,
    linear-gradient(rgb(37, 37, 37), rgb(37, 37, 37)) 100% 100% / 33.34% 33.34% no-repeat,
    rgb(211, 211, 211) !important;
}`;

interface PresentationFrameImage {
  getSize(): { width: number; height: number };
  toBitmap(): Uint8Array;
}

export function frameContainsWindowPresentationMarker(frame: unknown): boolean {
  if (!isPresentationFrameImage(frame)) {
    throw new Error('Application presentation frame was unavailable.');
  }
  const size = frame.getSize();
  if (
    !Number.isInteger(size.width) ||
    !Number.isInteger(size.height) ||
    size.width < 3 ||
    size.height < 3
  ) {
    throw new Error('Application presentation frame was unavailable.');
  }
  const bitmap = frame.toBitmap();
  if (!(bitmap instanceof Uint8Array)) {
    throw new Error('Application presentation frame was unavailable.');
  }
  const requiredBytes = size.width * size.height * 4;
  if (bitmap.byteLength < requiredBytes) {
    throw new Error('Application presentation frame was unavailable.');
  }
  const positions = [1 / 6, 1 / 2, 5 / 6];
  const expected = [
    PRESENTATION_MARKER_DARK,
    PRESENTATION_MARKER_LIGHT,
    PRESENTATION_MARKER_DARK,
    PRESENTATION_MARKER_LIGHT,
    PRESENTATION_MARKER_DARK,
    PRESENTATION_MARKER_LIGHT,
    PRESENTATION_MARKER_DARK,
    PRESENTATION_MARKER_LIGHT,
    PRESENTATION_MARKER_DARK,
  ];
  let sampleIndex = 0;
  for (const y of positions) {
    for (const x of positions) {
      const pixelX = Math.min(size.width - 1, Math.floor(size.width * x));
      const pixelY = Math.min(size.height - 1, Math.floor(size.height * y));
      const offset = ((pixelY * size.width) + pixelX) * 4;
      const target = expected[sampleIndex]!;
      sampleIndex += 1;
      if (
        Math.abs(bitmap[offset]! - target) > PRESENTATION_MARKER_TOLERANCE ||
        Math.abs(bitmap[offset + 1]! - target) > PRESENTATION_MARKER_TOLERANCE ||
        Math.abs(bitmap[offset + 2]! - target) > PRESENTATION_MARKER_TOLERANCE ||
        bitmap[offset + 3]! < 240
      ) {
        return false;
      }
    }
  }
  return true;
}

function isPresentationFrameImage(value: unknown): value is PresentationFrameImage {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const candidate = value as Partial<PresentationFrameImage>;
  return typeof candidate.getSize === 'function' && typeof candidate.toBitmap === 'function';
}

export interface WindowPresentationAdapter {
  getAuthoritativeStatus(): LauncherRootAuthoritativeStatus;
  schedule(callback: () => void, delayMs: number): unknown;
  clearScheduled(handle: unknown): void;
  subscribeToPresentationFrames(callback: (frame: unknown) => void): void;
  unsubscribeFromPresentationFrames(): void;
  insertPresentationMarker(
    onInserted: (markerHandle: unknown) => void,
    onUnavailable: () => void
  ): void;
  removePresentationMarker(
    markerHandle: unknown,
    onRemoved: () => void,
    onUnavailable: () => void
  ): void;
  invalidatePresentationFrame(): void;
  frameContainsPresentationMarker(frame: unknown): boolean;
  showWindow(): void;
  focusWindow(): void;
  reportFocusUnavailable(): void;
  sendVisibilityTimeout(): void;
  showNativeFatal(): void;
  destroyWindow(): void;
  quitApplication(): void;
}

export interface WindowPresentationDeadlines {
  presentationDeadlineMs: number;
  fallbackGraceMs: number;
}

export interface RendererPresentationCommit {
  currentTopDocument: boolean;
  presentation: unknown;
}

export type RendererPresentationCommitOutcome =
  | { status: 'accepted' }
  | { status: 'invalid' }
  | {
      status: 'ignored';
      reason:
        | 'terminal'
        | 'stale-document'
        | 'authority-mismatch'
        | 'already-committed'
        | 'fallback-required';
    };

export interface WindowPresentationOwner {
  browserReady(): void;
  documentChanged(): void;
  documentReady(): void;
  dispose(): void;
  focusRequested(): WindowFocusRequestOutcome;
  loadFailed(): void;
  preloadFailed(): void;
  rendererCommitted(commit: RendererPresentationCommit): RendererPresentationCommitOutcome;
}

export type WindowFocusRequestOutcome =
  | { status: 'queued' }
  | { status: 'focused' }
  | { status: 'unavailable' }
  | { status: 'ignored' };

export function createWindowPresentationOwner(
  adapter: WindowPresentationAdapter,
  deadlines: WindowPresentationDeadlines
): WindowPresentationOwner {
  let browserReady = false;
  let documentReady = false;
  let rendererReady = false;
  let committedPresentation: LauncherRootCommittedPresentation | undefined;
  let preloadFailed = false;
  let focusPending = false;
  let phase: 'awaiting' | 'fallback' | 'shown' | 'fatal' | 'disposed' = 'awaiting';
  let deadline: unknown;
  let fallbackGrace: unknown;
  let frameProof = false;
  let nextFrameChallengeId = 1;
  let frameChallenge: FrameChallenge | undefined;

  function ownsFrameChallenge(challenge: FrameChallenge): boolean {
    return frameChallenge?.id === challenge.id;
  }

  function clearOwnedTimer(handle: unknown): void {
    if (handle === undefined) {
      return;
    }
    try {
      adapter.clearScheduled(handle);
    } catch {
      // A bounded timer callback observes terminal phase if native clearing fails.
    }
  }

  function stopFrameSubscription(
    challenge: FrameChallenge,
    failureIsFatal: boolean
  ): boolean {
    if (!challenge.subscriptionActive) {
      return true;
    }
    challenge.subscriptionActive = false;
    try {
      adapter.unsubscribeFromPresentationFrames();
      return true;
    } catch {
      if (failureIsFatal) {
        terminateFatally();
      }
      return false;
    }
  }

  function finishCancelledFrameChallenge(challenge: FrameChallenge): void {
    if (!ownsFrameChallenge(challenge)) {
      return;
    }
    frameChallenge = undefined;
    if (phase === 'awaiting' || phase === 'fallback') {
      showWhenReady();
    }
  }

  function removeCancelledMarker(challenge: FrameChallenge): void {
    if (challenge.markerRemovalStarted || challenge.markerHandle === undefined) {
      return;
    }
    challenge.markerRemovalStarted = true;
    try {
      adapter.removePresentationMarker(
        challenge.markerHandle,
        () => {
          if (challenge.markerRemovalSettled) {
            return;
          }
          challenge.markerRemovalSettled = true;
          finishCancelledFrameChallenge(challenge);
        },
        () => {
          if (challenge.markerRemovalSettled) {
            return;
          }
          challenge.markerRemovalSettled = true;
          if (phase === 'awaiting' || phase === 'fallback') {
            terminateFatally();
          } else {
            finishCancelledFrameChallenge(challenge);
          }
        }
      );
    } catch {
      if (challenge.markerRemovalSettled) {
        return;
      }
      challenge.markerRemovalSettled = true;
      if (phase === 'awaiting' || phase === 'fallback') {
        terminateFatally();
      } else {
        finishCancelledFrameChallenge(challenge);
      }
    }
  }

  function cancelFrameChallenge(failureIsFatal = true): boolean {
    const challenge = frameChallenge;
    frameProof = false;
    if (!challenge) {
      return phase !== 'fatal' && phase !== 'disposed';
    }
    challenge.active = false;
    if (!stopFrameSubscription(challenge, failureIsFatal)) {
      return phase !== 'fatal' && phase !== 'disposed';
    }
    if (!challenge.insertionSettled) {
      return phase !== 'fatal' && phase !== 'disposed';
    }
    if (challenge.markerHandle !== undefined && !challenge.markerRemovalSettled) {
      removeCancelledMarker(challenge);
      return phase !== 'fatal' && phase !== 'disposed';
    }
    finishCancelledFrameChallenge(challenge);
    return phase !== 'fatal' && phase !== 'disposed';
  }

  function terminateFatally(): void {
    if (phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
      return;
    }
    phase = 'fatal';
    cancelFrameChallenge(false);
    clearOwnedTimer(deadline);
    clearOwnedTimer(fallbackGrace);
    try {
      adapter.showNativeFatal();
    } catch {
      // Native diagnostics are best-effort; cleanup remains mandatory.
    }
    try {
      adapter.destroyWindow();
    } catch {
      // Application termination remains the final owner action.
    }
    try {
      adapter.quitApplication();
    } catch {
      // No further in-process recovery is truthful after a fatal terminal.
    }
  }

  function beginFallback(): void {
    if (phase !== 'awaiting') {
      return;
    }
    phase = 'fallback';
    rendererReady = false;
    committedPresentation = undefined;
    if (!cancelFrameChallenge()) {
      return;
    }
    try {
      const scheduledFallbackGrace = adapter.schedule(
        terminateFatally,
        deadlines.fallbackGraceMs
      );
      if (phase !== 'fallback') {
        clearOwnedTimer(scheduledFallbackGrace);
        return;
      }
      fallbackGrace = scheduledFallbackGrace;
      adapter.sendVisibilityTimeout();
    } catch {
      terminateFatally();
    }
  }

  try {
    const scheduledDeadline = adapter.schedule(
      beginFallback,
      deadlines.presentationDeadlineMs
    );
    if (phase === 'awaiting') {
      deadline = scheduledDeadline;
    } else {
      clearOwnedTimer(scheduledDeadline);
    }
  } catch {
    terminateFatally();
  }

  function tryFocusWindow(): WindowFocusRequestOutcome {
    try {
      adapter.focusWindow();
      return { status: 'focused' };
    } catch {
      try {
        adapter.reportFocusUnavailable();
      } catch {
        // The window is already visible; diagnostic failure must not hide it.
      }
      return { status: 'unavailable' };
    }
  }

  function frameChallengeFailed(challenge: FrameChallenge): void {
    if (!ownsFrameChallenge(challenge) || !challenge.active) {
      return;
    }
    terminateFatally();
  }

  function invalidateFrame(challenge: FrameChallenge): void {
    if (frameChallenge !== challenge || !challenge.active) {
      return;
    }
    try {
      adapter.invalidatePresentationFrame();
    } catch {
      frameChallengeFailed(challenge);
    }
  }

  function beginActiveMarkerRemoval(challenge: FrameChallenge): void {
    if (
      !ownsFrameChallenge(challenge) ||
      !challenge.active ||
      challenge.markerRemovalStarted ||
      challenge.markerHandle === undefined
    ) {
      return;
    }
    challenge.markerRemovalStarted = true;
    challenge.stage = 'removing-marker';
    try {
      adapter.removePresentationMarker(
        challenge.markerHandle,
        () => {
          if (challenge.markerRemovalSettled) {
            return;
          }
          challenge.markerRemovalSettled = true;
          if (!ownsFrameChallenge(challenge)) {
            return;
          }
          if (!challenge.active) {
            finishCancelledFrameChallenge(challenge);
            return;
          }
          challenge.stage = 'awaiting-marker-free-frame';
          invalidateFrame(challenge);
        },
        () => {
          if (challenge.markerRemovalSettled) {
            return;
          }
          challenge.markerRemovalSettled = true;
          frameChallengeFailed(challenge);
        }
      );
    } catch {
      frameChallengeFailed(challenge);
    }
  }

  function completeFrameChallenge(challenge: FrameChallenge): void {
    if (!ownsFrameChallenge(challenge) || !challenge.active) {
      return;
    }
    challenge.active = false;
    if (!stopFrameSubscription(challenge, true) || phase === 'fatal') {
      return;
    }
    frameChallenge = undefined;
    frameProof = true;
    showWhenReady();
  }

  function handlePresentationFrame(challenge: FrameChallenge, frame: unknown): void {
    if (!ownsFrameChallenge(challenge) || !challenge.active) {
      return;
    }
    challenge.frameSequence += 1;
    let containsMarker: boolean;
    try {
      containsMarker = adapter.frameContainsPresentationMarker(frame);
    } catch {
      frameChallengeFailed(challenge);
      return;
    }
    if (
      challenge.stage === 'awaiting-marker-frame' &&
      containsMarker
    ) {
      challenge.markerFrameSequence = challenge.frameSequence;
      challenge.markerObserved = true;
      beginActiveMarkerRemoval(challenge);
      return;
    }
    if (
      challenge.stage === 'awaiting-marker-free-frame' &&
      !containsMarker &&
      challenge.markerFrameSequence !== undefined &&
      challenge.frameSequence > challenge.markerFrameSequence
    ) {
      completeFrameChallenge(challenge);
    }
  }

  function startFrameChallenge(): void {
    if (frameChallenge || frameProof || phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
      return;
    }
    const challenge: FrameChallenge = {
      id: nextFrameChallengeId,
      active: true,
      insertionSettled: false,
      markerObserved: false,
      markerRemovalStarted: false,
      markerRemovalSettled: false,
      subscriptionActive: true,
      frameSequence: 0,
      stage: 'inserting-marker',
    };
    nextFrameChallengeId += 1;
    frameChallenge = challenge;
    try {
      adapter.subscribeToPresentationFrames((frame) => {
        handlePresentationFrame(challenge, frame);
      });
    } catch {
      challenge.subscriptionActive = false;
      frameChallengeFailed(challenge);
      return;
    }
    try {
      adapter.insertPresentationMarker(
        (markerHandle) => {
          if (challenge.insertionSettled) {
            return;
          }
          challenge.insertionSettled = true;
          challenge.markerHandle = markerHandle;
          if (!challenge.active) {
            removeCancelledMarker(challenge);
            return;
          }
          challenge.stage = 'awaiting-marker-frame';
          if (challenge.markerObserved) {
            beginActiveMarkerRemoval(challenge);
          } else {
            invalidateFrame(challenge);
          }
        },
        () => {
          if (challenge.insertionSettled) {
            return;
          }
          challenge.insertionSettled = true;
          if (!challenge.active) {
            finishCancelledFrameChallenge(challenge);
            return;
          }
          frameChallengeFailed(challenge);
        }
      );
    } catch {
      if (!challenge.insertionSettled) {
        challenge.insertionSettled = true;
      }
      frameChallengeFailed(challenge);
    }
  }

  function showWhenReady(): void {
    if (
      phase === 'shown' ||
      phase === 'fatal' ||
      phase === 'disposed' ||
      !browserReady ||
      !documentReady ||
      (!rendererReady && !preloadFailed)
    ) {
      return;
    }
    if (!preloadFailed && committedPresentation !== 'bridge-unavailable') {
      let authoritativeStatus: LauncherRootAuthoritativeStatus;
      try {
        authoritativeStatus = adapter.getAuthoritativeStatus();
      } catch {
        terminateFatally();
        return;
      }
      if (committedPresentation !== authoritativeStatus) {
        rendererReady = false;
        committedPresentation = undefined;
        cancelFrameChallenge();
        return;
      }
    }
    if (!frameProof) {
      startFrameChallenge();
      return;
    }
    try {
      adapter.showWindow();
    } catch {
      terminateFatally();
      return;
    }
    phase = 'shown';
    clearOwnedTimer(deadline);
    clearOwnedTimer(fallbackGrace);
    if (focusPending) {
      focusPending = false;
      tryFocusWindow();
    }
  }

  return {
    browserReady() {
      if (phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
        return;
      }
      browserReady = true;
      showWhenReady();
    },
    documentChanged() {
      if (phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
        return;
      }
      browserReady = false;
      documentReady = false;
      rendererReady = false;
      committedPresentation = undefined;
      preloadFailed = false;
      cancelFrameChallenge();
    },
    documentReady() {
      if (phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
        return;
      }
      documentReady = true;
      showWhenReady();
    },
    dispose() {
      if (phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
        return;
      }
      phase = 'disposed';
      cancelFrameChallenge(false);
      clearOwnedTimer(deadline);
      clearOwnedTimer(fallbackGrace);
    },
    focusRequested() {
      if (phase === 'shown') {
        return tryFocusWindow();
      } else if (phase !== 'fatal' && phase !== 'disposed') {
        focusPending = true;
        return { status: 'queued' };
      }
      return { status: 'ignored' };
    },
    loadFailed() {
      terminateFatally();
    },
    preloadFailed() {
      if (phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
        return;
      }
      rendererReady = false;
      committedPresentation = undefined;
      preloadFailed = true;
      cancelFrameChallenge();
      showWhenReady();
    },
    rendererCommitted(commit) {
      if (phase === 'shown' || phase === 'fatal' || phase === 'disposed') {
        return { status: 'ignored', reason: 'terminal' };
      }
      if (!isLauncherRootCommittedPresentation(commit.presentation)) {
        return { status: 'invalid' };
      }
      if (!commit.currentTopDocument) {
        return { status: 'ignored', reason: 'stale-document' };
      }
      if (rendererReady) {
        return { status: 'ignored', reason: 'already-committed' };
      }
      if (phase === 'fallback' && commit.presentation !== 'bridge-unavailable') {
        return { status: 'ignored', reason: 'fallback-required' };
      }
      if (commit.presentation !== 'bridge-unavailable') {
        let authoritativeStatus: LauncherRootAuthoritativeStatus;
        try {
          authoritativeStatus = adapter.getAuthoritativeStatus();
        } catch {
          terminateFatally();
          return { status: 'ignored', reason: 'terminal' };
        }
        if (commit.presentation !== authoritativeStatus) {
          return { status: 'ignored', reason: 'authority-mismatch' };
        }
      }
      rendererReady = true;
      committedPresentation = commit.presentation;
      showWhenReady();
      return { status: 'accepted' };
    },
  };
}

type FrameChallengeStage =
  | 'inserting-marker'
  | 'awaiting-marker-frame'
  | 'removing-marker'
  | 'awaiting-marker-free-frame';

interface FrameChallenge {
  id: number;
  active: boolean;
  insertionSettled: boolean;
  markerHandle?: unknown;
  markerObserved: boolean;
  markerRemovalStarted: boolean;
  markerRemovalSettled: boolean;
  subscriptionActive: boolean;
  frameSequence: number;
  markerFrameSequence?: number;
  stage: FrameChallengeStage;
}

function isLauncherRootCommittedPresentation(
  value: unknown
): value is LauncherRootCommittedPresentation {
  return value === 'ready' ||
    value === 'recovery-required' ||
    value === 'bridge-unavailable';
}
