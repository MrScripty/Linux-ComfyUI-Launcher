import { configDefaults, defineConfig } from 'vitest/config';
import base from './vitest.config';

// Deliberately selected real producer/preload/renderer gate. Ordinary unit
// runs do not silently skip it or generate a substitute fixture.
export default defineConfig({
  ...base,
  test: {
    ...base.test,
    include: ['conformance/**/*.test.tsx'],
    exclude: configDefaults.exclude,
  },
});
