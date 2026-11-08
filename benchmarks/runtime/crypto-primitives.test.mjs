import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {createHash} from 'node:crypto';import {cryptoIDs,cryptoChecksum,assertCryptoSource,assertCryptoReference} from './crypto-primitives-contract.mjs';const rows=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url))).workloads.filter(row=>row.family==='crypto-primitives');test('all crypto authored peers preserve exact source and independent observations',()=>{assert.equal(rows.length,29);assert.equal(new Set(cryptoIDs).size,29);for(const row of rows){for(const engine of ['native','typescript']){const source=fs.readFileSync(new URL(row[engine==='native'?'source':'counterpart'],import.meta.url),'utf8');assertCryptoSource(source,row,engine);const token=row.methodExpressions[engine][0];const changed=replaceCodeSnippet(source,token,'REMOVED');assert.notEqual(changed,source);const rehashed={...row,[engine==='native'?'sourceSha256':'counterpartSha256']:createHash('sha256').update(changed).digest('hex')};assert.throws(()=>assertCryptoSource(changed,rehashed,engine));}const rust=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assertCryptoReference(rust,row);const changed=rust.replace('function0','missing');assert.throws(()=>assertCryptoReference(changed,{...row,rustReferenceSha256:createHash('sha256').update(changed).digest('hex')}));for(const n of [0,1,2])for(const seed of [0,7,31])assert(Number.isSafeInteger(cryptoChecksum(row.id,n,seed)));}assert.equal(cryptoChecksum('crypto_random_int',2,7),132);assert.equal(cryptoChecksum('crypto_verify_hmac_64',1,7),131);assert.throws(()=>cryptoChecksum('unknown',1,7));});

test('allocation profiler retains crypto source proof and existing async adapter',()=>{
 const profiler=fs.readFileSync(new URL('./profile-native.mjs',import.meta.url),'utf8');
 assert.ok(profiler.includes("import {assertCryptoSource,assertCryptoReference,cryptoEvidence,freezeCryptoRuntime} from './crypto-primitives-contract.mjs'"));
 assert.equal(profiler.split('assertCryptoSource(').length-1,4);
 assert.equal(profiler.split('assertCryptoReference(').length-1,2);
 assert.ok(profiler.includes('freezeCryptoRuntime();assertCryptoSource('));
 assert.ok(profiler.includes("if(workload.family==='crypto-primitives')cryptoEvidence();return sample;"));
 assert.ok(profiler.includes('cryptoInputs:cryptoEvidence()'));
 assert.ok(profiler.includes("oracleInputs: ['crypto-primitives-contract.mjs'"));
 const adapters=fs.readFileSync(new URL('./adapters.mjs',import.meta.url),'utf8');
 assert.ok(adapters.includes('if(workload.cryptoAsync)return cryptoAsyncDriver(driver,workload,emitted,source,{reference,probe})'));
 const contract=fs.readFileSync(new URL('./crypto-primitives-contract.mjs',import.meta.url),'utf8');
 assert.ok(contract.includes('probe?asyncScalarAllocationDriver(driver,abi):asyncScalarRustDriver(abi)'));
});

test('all nine streaming workloads reject length interpreted as end even with refreshed hashes',()=>{
 const streams=rows.filter(row=>row.cryptoOperation.endsWith('_stream'));assert.equal(streams.length,9);
 for(const row of streams){const source=fs.readFileSync(new URL(row.source,import.meta.url),'utf8');assertCryptoSource(source,row,'native');const changed=replaceCodeSnippet(source,'view(input,offset,end)','view(input,offset,end-offset)');assert.notEqual(source,changed);assert.throws(()=>assertCryptoSource(changed,{...row,sourceSha256:createHash('sha256').update(changed).digest('hex')},'native'),/end index/);}
 const runner=fs.readFileSync(new URL('./run.mjs',import.meta.url),'utf8');assert.ok(runner.includes("['native-url','structured-logging','crypto-primitives'].includes(workloads[index]?.family)"));assert.ok(runner.includes("else if(w.family==='crypto-primitives'){freezeCryptoRuntime();assertCryptoSource(source,w,'native');assertCryptoSource(ts,w,'typescript');assertCryptoReference(hand,w);}"));
});

test('HKDF reference passes the existing info slice without an extra borrow',()=>{
 const hkdf=rows.filter(row=>row.cryptoOperation==='hkdf');assert.equal(hkdf.length,3);
 for(const row of hkdf){const source=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assertCryptoReference(source,row);const changed=source.replace('.expand(info,','.expand(&info,');assert.notEqual(changed,source);assert.throws(()=>assertCryptoReference(changed,{...row,rustReferenceSha256:createHash('sha256').update(changed).digest('hex')}),/info must be the byte slice/);}
});


test('all four password references verify the PHC str slice after rehashing',()=>{
 const passwords=rows.filter(row=>row.cryptoAsync);assert.equal(passwords.length,4);
 for(const row of passwords){const source=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assertCryptoReference(source,row);const changed=source.replace('verify_password(&password,encoded.as_str())','verify_password(&password,&encoded)');assert.notEqual(changed,source);assert.throws(()=>assertCryptoReference(changed,{...row,rustReferenceSha256:createHash('sha256').update(changed).digest('hex')}),/PHC str slice/);}
});
