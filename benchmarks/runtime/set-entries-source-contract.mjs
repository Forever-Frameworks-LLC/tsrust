import {createHash} from 'node:crypto';
export function assertSetEntriesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='set_entries'||row.family!=='set-entries')throw Error('Set entry family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Set entry fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete set entry scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Set entry denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of [`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,`new Set<${integer}>([i+seed,i+seed+1,i+seed])`,'values.delete(i+seed)','values.add(i+seed)','values.clear();','new Set<string>([String(input),"tail"])','names.clear();','entries[0][0]','entries[1][1]','labels[0][0]','labels[0][1]','return output;'])if(!source.includes(text))throw Error('Set entry scenario: '+text);
 for(const receiver of ['values','names']){const call=`${receiver}.entries()`;if(!source.includes(kind==='native'?call:`Array.from(${call})`))throw Error('Set entry operation: '+receiver);}
}
