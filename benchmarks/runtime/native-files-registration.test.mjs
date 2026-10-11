import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {createNativeFileFixture} from './native-files-input.mjs';
import {expectedNativeFilesValues, expectedNativeFilesObservation} from './native-files-oracle.mjs';
import {assertNativeFilesSource} from './native-files-source-contract.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
import {spawnSync} from 'node:child_process';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
test('Runtime file inputs disappear on normal and failed process exit',()=>{
  const row=manifest.workloads.find(row=>row.id==='native_file_read_close');
  const module=new URL('./native-files-input.mjs',import.meta.url).href;
  for(const fail of [false,true]) {
    const script=`import {nativeFileInput} from ${JSON.stringify(module)};console.log(nativeFileInput(${JSON.stringify(row)}));${fail?'throw Error("controlled failure");':''}`;
    const child=spawnSync(process.execPath,['--input-type=module','-e',script],{encoding:'utf8',timeout:10000});
    assert.equal(child.status,fail?1:0);
    const path=child.stdout.trim();
    assert.ok(path.endsWith('/input.bin'));
    assert.equal(fs.existsSync(path),false);
  }
});
for (const id of ['native_file_read_close', 'native_file_scope_drop']) test(`${id} retains matched I/O, independent observations and isolated input`, () => {
  const row = manifest.workloads.find(row => row.id === id);
  assert.ok(row);
  assert.equal(reportCategory(id), 'resources');
  assert.equal(row.measured, true);
    assert.equal(row.measurementStatus, 'measured-resources-bytes-family');
    assert.equal(row.measurementEvidence, 'runtime/results/s07-measurement-proof-2026-10-11.json');
  assert.throws(()=>createNativeFileFixture({...row,fixtureSha256:'changed'}));
  const fixture = createNativeFileFixture(row);
  try {
    const bytes = fs.readFileSync(fixture.path);
    assert.equal(bytes.length, 4096);
    assert.equal(createHash('sha256').update(bytes).digest('hex'), row.fixtureSha256);
    assert.deepEqual(expectedNativeFilesValues(id, 3, 0, row.inputText), [[1,48],[2,97],[3,147]]);
    for (const n of [0,1,37,2048]) for (const seed of [0,7,1000000]) {
      const expected = expectedNativeFilesObservation(id,n,seed,row.inputText);
      assert.deepEqual(expectedExactResult(id,n,seed,row.inputText), expected);
      assert.equal(expectedChecksum(id,n,seed,row.inputText),expected.checksum);
      const independent = expectedNativeFilesValues(id,n,seed,row.inputText);
      for (let i=0;i<n;i++) {
        const count=(i+seed)%64+1;
        assert.deepEqual(independent[i],[count,[...bytes.subarray(0,count)].reduce((sum,value)=>sum+value,0)]);
      }
    }
  } finally { fixture.cleanup(); }
  assert.equal(fs.existsSync(fixture.path), false);
  const emitted='fn tsr_result_function2(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>, TsrFunctionError2> { todo!() }\nfn main() {}';
  assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,true);
  for (const [kind,file] of [['native',row.source],['typescript',row.counterpart]]) {
    const source=read('./'+file);
    assertNativeFilesSource(source,row,kind);
    assert.throws(()=>assertNativeFilesSource(source,{...row,operationsPerIteration:2},kind));
    const mutations=['const count = (i + seed) % 64 + 1;',kind==='native'?'read(file, count)':'fileSystem.readSync(file, bytes, 0, count, null)'];
    if(kind==='typescript')mutations.push('fileSystem.closeSync(file);');
    else if(id==='native_file_read_close')mutations.push('close(file)');
    for(const snippet of mutations) {
      const changed=replaceCodeSnippet(source,snippet,'');
      const hash=kind==='native'?'sourceSha256':'counterpartSha256';
      assert.throws(()=>assertNativeFilesSource(changed,{...row,[hash]:createHash('sha256').update(changed).digest('hex')},kind));
    }
  }
  for(const file of ['./run.mjs','./profile-native.mjs']) {
    const source=read(file);
    assert.ok(source.includes('assertNativeFilesSource(source'));
    assert.ok(source.includes('nativeFileInput('));
    for(const name of ['native-files-oracle.mjs','native-files-input.mjs','native-files-source-contract.mjs'])assert.ok(source.includes("'"+name+"'"));
  }
});
