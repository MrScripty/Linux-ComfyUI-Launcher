import { createHash } from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';

const MAX_MARKER_BYTES = 1024;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

/**
 * Scope only display caches, never filesystem access or recovery capabilities.
 * Missing/unsupported identity means no cache, not a different library or a
 * startup error. Physical identity deliberately invalidates caches after a
 * root's observed identity changes even when its logical marker is identical.
 */
export function readLibraryDisplayScope(launcherRoot: string): string | null {
  try {
    return readDisplayScope(launcherRoot);
  } catch {
    // This includes descriptor cleanup failure: cache availability must never
    // prevent the selected library from starting.
    return null;
  }
}

function readDisplayScope(launcherRoot: string): string | null {
  let descriptor: number | undefined;
  try {
    const root = fs.realpathSync(path.join(launcherRoot, 'shared-resources', 'models'));
    const rootIdentity = fs.statSync(root, { bigint: true });
    if (!rootIdentity.isDirectory()) return null;
    const marker = path.join(root, '.pumas-library-id.json');
    const before = fs.lstatSync(marker, { bigint: true });
    if (!before.isFile() || before.size > BigInt(MAX_MARKER_BYTES)) return null;
    descriptor = fs.openSync(marker, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW | fs.constants.O_NONBLOCK);
    const opened = fs.fstatSync(descriptor, { bigint: true });
    if (!opened.isFile() || opened.dev !== before.dev || opened.ino !== before.ino) return null;
    const bytes = Buffer.alloc(MAX_MARKER_BYTES + 1);
    let count = 0;
    while (count < bytes.length) {
      const read = fs.readSync(descriptor, bytes, count, bytes.length - count, null);
      if (read === 0) break;
      count += read;
    }
    if (count > MAX_MARKER_BYTES) return null;
    const serialized = new TextDecoder('utf-8', { fatal: true }).decode(bytes.subarray(0, count));
    const document: unknown = JSON.parse(serialized);
    if (typeof document !== 'object' || document === null || Array.isArray(document)) return null;
    const fields = Object.entries(document);
    if (fields.length !== 2) return null;
    const version = fields.find(([key]) => key === 'schema_version')?.[1];
    const id: unknown = fields.find(([key]) => key === 'library_id')?.[1];
    if (version !== 1 || typeof id !== 'string' || !UUID.test(id) || id === '00000000-0000-0000-0000-000000000000') return null;
    // The core writer emits this two-field document. Admit only its compact or
    // pretty serialization for display caching, so JSON.parse cannot hide
    // duplicate fields. Other core-readable representations simply get no cache.
    const canonical = { schema_version: 1, library_id: id };
    if (serialized.trim() !== JSON.stringify(canonical) &&
        serialized.trim() !== JSON.stringify(canonical, null, 2)) return null;
    const after = fs.statSync(root, { bigint: true });
    if (after.dev !== rootIdentity.dev || after.ino !== rootIdentity.ino) return null;
    return `display-v1:${createHash('sha256').update(JSON.stringify([
      root, rootIdentity.dev.toString(), rootIdentity.ino.toString(), id,
    ])).digest('hex')}`;
  } finally {
    if (descriptor !== undefined) fs.closeSync(descriptor);
  }
}
