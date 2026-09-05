import { build } from 'esbuild';

// tsc emits the generated TypeScript wrapper but not its JavaScript input.
// Supply its Node-side module in the shell's CommonJS output cohort as well.
await build({
  entryPoints: ['src/generated/desktop-contract.validators.js'],
  outfile: 'dist/generated/desktop-contract.validators.js',
  format: 'cjs',
  platform: 'node',
  target: 'node22',
});

// Sandboxed Electron preload cannot require sibling Node modules. Keep the
// accepted sandbox and bundle only this process entry point's dependencies.
await build({
  entryPoints: ['src/preload.ts'],
  outfile: 'dist/preload.js',
  bundle: true,
  platform: 'browser',
  format: 'cjs',
  target: 'chrome142',
  external: ['electron'],
  sourcemap: true,
});
