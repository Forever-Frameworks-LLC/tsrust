import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { test } from 'node:test';
const root=new URL('../../',import.meta.url);
test('registered history workload preserves SQL bytes, iterations and timing protocol',()=>{
 const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url),'utf8'));
 for(const [name,digest] of Object.entries(manifest.hashes))assert.equal(createHash('sha256').update(fs.readFileSync(new URL(name,root))).digest('hex'),digest);
 for(const name of ['history.tsr','history.ts']){
  const source=fs.readFileSync(new URL(name,import.meta.url),'utf8');
  assert.match(source,name.endsWith('.tsr')?/index < 32; index\+\+\)\s*\{\s*await pool.migrate\(/:/index < 32; index\+\+\) migrate\(\)/);
  for(const pattern of [/index < 32/,/warmup < 8/,/sample < 31/,/SELECT id FROM service_notes/])assert.match(source,pattern);
 }
 const runner=fs.readFileSync(new URL('run.mjs',import.meta.url),'utf8');assert.match(runner,/verify\?1:3/);assert.match(runner,/const expected=1/);
});
