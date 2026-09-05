import assert from 'node:assert/strict';
import test from 'node:test';
import { schemaType, generate } from './generate-desktop-contract.mjs';

test('type projection preserves optional null and tagged alternatives', () => {
  assert.equal(schemaType({type:'object', required:['state'], properties:{state:{enum:['partial']}, progress:{type:['number','null']}}}), '{ "state": "partial"; "progress"?: number | null }');
});

test('type projection refuses unknown reachable schema constructs', () => {
  assert.throws(() => schemaType({type:'array',items:[{type:'string'}]}), /Unsupported/);
  assert.throws(() => schemaType({dynamicRef:'#anchor'}), /Unsupported/);
});

test('standalone validator preserves UTF8 byte bounds and closed shapes', async () => {
  const files = await generate({format:'pumas-desktop-contract-1',dialect:'http://json-schema.org/draft-07/schema#',schemas:{Example:{type:'object',additionalProperties:false,required:['text'],properties:{text:{type:'string',pumasUtf8Max:4}}}}});
  const module = await import(`data:text/javascript;base64,${Buffer.from(files['desktop-contract.validators.js']).toString('base64')}`);
  assert.equal(module.validateExample({text:'éé'}), true);
  assert.equal(module.validateExample({text:'ééé'}), false);
  assert.equal(module.validateExample({text:'ok', extra:true}), false);
  assert.equal(module.validateExample({text:4}), false);
});

test('portable recovery paths reject unsafe prefixes and preserve Unicode bytes', async () => {
  const files = await generate({format:'pumas-desktop-contract-1',dialect:'http://json-schema.org/draft-07/schema#',schemas:{Path:{type:'string',pumasPortablePath:true,pumasUtf8Max:4096}}});
  const module = await import(`data:text/javascript;base64,${Buffer.from(files['desktop-contract.validators.js']).toString('base64')}`);
  assert.equal(module.validatePath('llm/example/model'), true);
  assert.equal(module.validatePath('llm/例/模型'), true);
  for (const path of ['', '/absolute','../escape','folder/CON.gguf','folder\\file','folder/file.',`folder/${'é'.repeat(128)}`]) assert.equal(module.validatePath(path), false, path);
});

test('canonical text uses Unicode White_Space rather than JavaScript trim', async () => {
  const files = await generate({format:'pumas-desktop-contract-1',dialect:'http://json-schema.org/draft-07/schema#',schemas:{Text:{type:'string',pumasCanonicalText:true}}});
  const module = await import(`data:text/javascript;base64,${Buffer.from(files['desktop-contract.validators.js']).toString('base64')}`);
  assert.equal(module.validateText('Name'), true);
  assert.equal(module.validateText('   '), false);
  assert.equal(module.validateText('\u0085Name'), false);
  assert.equal(module.validateText('\ufeffName'), true);
});
