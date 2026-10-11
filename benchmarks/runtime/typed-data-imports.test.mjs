import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertDataImportSource,dataImportChecksum,prepareDataImport} from './typed-data-imports.mjs';
import {expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
test('typed data imports pin source, input, operations and independent oracle',()=>{
 const row=JSON.parse(read('./manifest.json')).workloads.find(w=>w.id==='native_json_import');
 assert.ok(row);assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'json');
 for(const [kind,key] of [['native','source'],['typescript','counterpart']]){
  const source=read('./'+row[key]);assertDataImportSource(source,row,kind);
  assert.throws(()=>assertDataImportSource(source+' ',row,kind));
  const changed=source.replace('i < n','i < 1');
  assert.throws(()=>assertDataImportSource(changed,{...row,[kind==='native'?'sourceSha256':'counterpartSha256']:createHash('sha256').update(changed).digest('hex')},kind));
 }
 for(const n of [0,1,2,17,2000])for(const seed of [0,7,19])assert.equal(expectedChecksum(row.id,n,seed),seed+n*42);
 assert.throws(()=>dataImportChecksum(-1,0));
 assert.throws(()=>prepareDataImport({...row,dataSha256:'bad'},'/tmp'));
});
