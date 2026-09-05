import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath, URL } from 'node:url';
import test from 'node:test';
import { build } from 'esbuild';

const fixturePath = process.env.PUMAS_DESKTOP_CONTRACT_FIXTURES;
if (!fixturePath) throw new Error('Run this integration test through with-desktop-contract-fixtures.mjs.');
const fixtures = JSON.parse(await readFile(fixturePath, 'utf8'));
const compiled = await build({entryPoints:[fileURLToPath(new URL('../src/generated/desktop-contract.ts', import.meta.url))], bundle:true, format:'esm', platform:'browser', write:false});
const contract = await import(`data:text/javascript;base64,${Buffer.from(compiled.outputFiles[0].text).toString('base64')}`);

test('actual producer catalog and FTS cross the generated decoder', () => {
  const models = contract.decodeModelsOutcome(fixtures.models);
  const search = contract.decodeCatalogSearchOutcome(fixtures.search);
  assert.equal(models.status, 'valid');
  assert.equal(search.status, 'valid');
  assert.equal(Object.isFrozen(models.value.models), true);
  for (const model of search.value.models) assert.deepEqual(models.value.models[model.id], model);
  assert.deepEqual(search.value.models.map(model => model.id), ['llm/example/complete','llm/example/partial','llm/example/duplicate','llm/example/duplicate-peer']);
});

test('producer-impossible omitted null and missing explicit-null fields reject', () => {
  const changed = structuredClone(fixtures.models);
  changed.models['llm/example/complete'].format = null;
  assert.equal(contract.decodeModelsOutcome(changed).status, 'invalid');
  assert.equal(contract.decodePartialDownloadOutcome({success:true,action:'resume',download_id:'id',status:'queued',reason_code:null}).status, 'invalid');
});

test('map identity, duplicate consistency and partial progress cannot be weakened', () => {
  for (const mutate of [
    value => { value.models['llm/example/complete'].id = 'other'; },
    value => { value.models['llm/example/duplicate'].integrity.count = 9; },
    value => { value.models['llm/example/partial'].artifact.downloadProgressFraction = 1; },
    value => { value.models['llm/example/partial'].artifact.reasons = []; },
    value => { value.models['llm/example/complete'].sizeBytes = 9007199254740992; },
    value => { value.models['llm/example/complete'].displayName = '   '; },
    value => { value.models['llm/example/partial'].artifact.recovery.selectedArtifactId = ' '; },
    value => { value.models['llm/example/partial'].artifact.recovery.selectedArtifactId = 'é'.repeat(2049); },
    value => { value.models['llm/example/partial'].artifact.recovery.repoId = 'owner/..'; },
  ]) {
    const changed = structuredClone(fixtures.models);
    mutate(changed);
    assert.equal(contract.decodeModelsOutcome(changed).status, 'invalid');
  }
});

test('decoder retains no mutable alias and refuses non-JSON effects', () => {
  const changed = structuredClone(fixtures.models);
  const accepted = contract.decodeModelsOutcome(changed);
  changed.models['llm/example/complete'].displayName = 'mutated';
  assert.equal(accepted.value.models['llm/example/complete'].displayName, 'complete');
  let calls = 0;
  assert.equal(contract.decodeModelsOutcome({get success() {calls++; return true;}}).status, 'invalid');
  assert.equal(calls, 0);
});

test('actual download and recovery outcomes preserve nulls, numeric bounds and action correlations', () => {
  for (const [name, value] of [
    ['DownloadStatusOutcome',fixtures.download_status], ['DownloadListOutcome',fixtures.download_list],
    ['DownloadStartedOutcome',fixtures.download_started], ['DownloadMutationOutcome',fixtures.download_mutation],
    ['PartialDownloadOutcome',fixtures.recovery_outcome], ['RecoverDownloadParams',fixtures.recovery_request],
    ['PartialDownloadOutcome',fixtures.recovery_busy_outcome],
  ]) assert.equal(contract[`decode${name}`](value).status, 'valid', name);
  for (const [key, value] of [['progress',1.01], ['downloadedBytes',9007199254740992], ['speed',-1], ['etaSeconds',Number.POSITIVE_INFINITY]]) {
    assert.equal(contract.decodeDownloadStatusOutcome({...fixtures.download_status,[key]:value}).status, 'invalid');
  }
  assert.equal(contract.decodePartialDownloadOutcome({...fixtures.recovery_outcome,status:'paused'}).status, 'invalid');
  const busy = contract.decodePartialDownloadOutcome(fixtures.recovery_busy_outcome);
  assert.equal(busy.value.reason_code, 'download_root_busy');
  assert.equal(busy.value.success, false);
  assert.equal(busy.value.download_id, null);
  assert.equal(contract.decodePartialDownloadOutcome({...fixtures.recovery_busy_outcome,download_id:'unexpected-admission',status:'queued'}).status, 'invalid');
  assert.equal(contract.decodeDownloadMutationOutcome({success:false}).status, 'invalid');
  assert.equal(contract.decodeRecoverDownloadParams({...fixtures.recovery_request,modelId:'../escape'}).status, 'invalid');
  assert.equal(contract.decodeSearchCatalogParams({query:'',offset:4294967296}).status, 'invalid');
  assert.equal(contract.decodeDownloadStatusOutcome({...fixtures.download_status,retryLimit:4294967296}).status, 'invalid');
  for (const probe of fixtures.recovery_request_probes) assert.equal(contract.decodeRecoverDownloadParams(probe.request).status === 'valid', probe.accepted);
  for (const probe of fixtures.catalog_text_probes) {
    const changed = structuredClone(fixtures.models);
    changed.models['llm/example/complete'].displayName = probe.input;
    assert.equal(contract.decodeModelsOutcome(changed).status === 'valid', probe.input === probe.emitted, JSON.stringify(probe.input));
  }
});
