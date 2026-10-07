import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertMapSnapshotsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='map_snapshots'||row.family!=='map-snapshots')throw Error('Map snapshot family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Map snapshot fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete map snapshot scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Map snapshot denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of [`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,`new Map<${integer},string>([[i+seed,String(input)],[i+seed+1,"tail"]])`,'values.delete(i+seed)','values.set(i+seed,String(input))','values.clear();','keys.join(",")','labels.join("|")','pairs[0][0]','pairs[1][1]','return output;'])if(!containsCodeSnippet(source, text))throw Error('Map snapshot scenario: '+text);
 for(const method of ['keys','values','entries']){const call=`values.${method}()`;if(!containsCodeSnippet(source, kind==='native'?call:`Array.from(${call})`))throw Error('Map snapshot operation: '+method);}
}
