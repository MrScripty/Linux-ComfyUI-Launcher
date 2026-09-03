import type { RuntimeModelServeActionProps } from './RuntimeModelServeAction';

/** Build-time replacement that removes serving actions from library-only builds. */
export function RuntimeModelServeAction(_props: RuntimeModelServeActionProps) {
  return null;
}
