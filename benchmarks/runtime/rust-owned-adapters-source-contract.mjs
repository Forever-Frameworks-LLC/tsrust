import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {rustOwnedAdapterIds} from './rust-owned-adapters-oracle.mjs';
const hash=source=>createHash('sha256').update(source).digest('hex');
export function assertRustOwnedAdapterSource(source,workload,engine){
 if(workload.family!=='rust-owned-adapters'||!rustOwnedAdapterIds.includes(workload.id))throw Error('Unknown owned adapter source');
 if(hash(source)!==workload[engine==='native'?'sourceSha256':'counterpartSha256'])throw Error('Owned adapter workload source changed');
 for(const snippet of [...workload.sourceRequired[engine],...workload.methodExpressions[engine]])if(!containsCodeSnippet(source,snippet))throw Error('Owned adapter call/observer changed');
 if(workload.adapterSource!=='workloads/rust-owned-adapter.rs'||hash(fs.readFileSync(new URL(workload.adapterSource,import.meta.url)))!==workload.adapterSha256)throw Error('Native owned adapter changed');
 if(workload.rustReference!==`parity/${workload.id}.rs`||hash(fs.readFileSync(new URL(workload.rustReference,import.meta.url)))!==workload.rustReferenceSha256)throw Error('Direct Rust control changed');
}
