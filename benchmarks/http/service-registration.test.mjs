import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
test('service boundary keeps matched source identities and independent checksum',()=>{
 const manifest=JSON.parse(fs.readFileSync(path.join(root,'service-manifest.json'),'utf8'));
 assert.equal(manifest.expected,128*(32+7+2+1));
 for(const extension of ['tsr','ts']){
  const source=fs.readFileSync(path.join(root,`service-boundary.${extension}`));
  assert.equal(crypto.createHash('sha256').update(source).digest('hex'),manifest.sources[extension]);
  const text=source.toString();for(const witness of ['index < 128','warmup < 8','sample < 31','one two','é','Token mismatch','Form mismatch','Log mismatch','Clock rounding mismatch'])assert.ok(text.includes(witness),witness);
 }
});
