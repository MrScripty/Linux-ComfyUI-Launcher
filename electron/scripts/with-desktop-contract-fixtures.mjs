import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, URL } from 'node:url';

const [command, ...args] = process.argv.slice(2);
if (!command) throw new Error('A consumer verification command is required.');
const directory = await mkdtemp(join(tmpdir(), 'pumas-desktop-contract-'));
try {
  const fixture = join(directory, 'producer.json');
  const generator = fileURLToPath(new URL('./generate-desktop-contract.mjs', import.meta.url));
  const exported = spawnSync(process.execPath, [generator, '--fixtures', fixture], {stdio:'inherit'});
  if (exported.status !== 0) throw new Error('Actual producer fixture generation failed.');
  const consumer = spawnSync(command, args, {stdio:'inherit', env:{...process.env, PUMAS_DESKTOP_CONTRACT_FIXTURES:fixture}});
  if (consumer.error) throw consumer.error;
  process.exitCode = consumer.status ?? 1;
} finally {
  // Only this invocation's private temporary directory is owned for deletion.
  await rm(directory, {recursive:true, force:false});
}
