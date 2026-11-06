import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {networkAddressIDs,networkAddressChecksum,assertNetworkAddressSource,assertNetworkAddressReference} from './network-address-contract.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url)));
test('address guards and independent complete observations',()=>{for(const id of networkAddressIDs){const row=manifest.workloads.find(row=>row.id===id);for(const engine of ['native','typescript'])assertNetworkAddressSource(fs.readFileSync(new URL(engine==='native'?row.source:row.counterpart,import.meta.url),'utf8'),row,engine);assertNetworkAddressReference(fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8'),row);assert.equal(networkAddressChecksum(id,0,7,row.inputText),0);}assert.equal(networkAddressChecksum('net_parse_ip',2,7,'2001:db8::1'),1717);assert.equal(networkAddressChecksum('net_parse_address',2,7,'[2001:db8::1]:8080'),networkAddressChecksum('net_format_address',2,7,'[2001:db8::1]:8080'));});

import {checkDnsObservations} from './network-dns-oracle.mjs';
test('literal resolver post-filter matches independent Node backend',async()=>{await checkDnsObservations([{host:'127.0.0.1',family:'any',values:['127.0.0.1']},{host:'127.0.0.1',family:'ipv6',values:[]},{host:'::1',family:'any',values:['::1']},{host:'::1',family:'ipv4',values:[]}]);});

import {adaptRustDriver} from './adapters.mjs';
import {createHash} from 'node:crypto';
test('address hand references use the guarded reference ABI and optional scalar reads stay explicit',()=>{
 const driver=fs.readFileSync(new URL('driver.rs',import.meta.url),'utf8');
 for(const id of networkAddressIDs){
  const row=manifest.workloads.find(row=>row.id===id),reference=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8'),source=fs.readFileSync(new URL(row.source,import.meta.url),'utf8');
  const adapted=adaptRustDriver(driver,row,{emitted:reference,source,reference:true});
  assert.match(adapted,/function0\(\s*std::hint::black_box/);assert.ok(adapted.includes('&input'));
  assert.throws(()=>adaptRustDriver(driver,row,{emitted:reference.replace('fn function0(n:i64','fn wrong(n:i64'),source,reference:true}),/Address reference ABI/);
  if(id!=='net_parse_ip'){const changed=source.replace(/\(formatted\.codePointAt\(j\)\s*\?\?\s*0\)/,'unwrap(formatted.codePointAt(j))');assert.notEqual(changed,source);assert.throws(()=>assertNetworkAddressSource(changed,{...row,sourceSha256:createHash('sha256').update(changed).digest('hex')},'native'));}
 }
});
