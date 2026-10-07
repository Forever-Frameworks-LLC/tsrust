import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertSetForEachSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='set_for_each'||row.family!=='set-for-each')throw Error('Set callback family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Set callback fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete set callback scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Set callback denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of [`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,`new Set<${integer}>([i+seed,i+seed+1,i+seed])`,'values.forEach((value,key,source)=>','total=total+value+key+source.size','new Set<string>([String(input),"tail",String(input)])','names.forEach((value,key,source)=>','source.has(key)','text=text+value+"|"','return output;'])if(!containsCodeSnippet(source, text))throw Error('Set callback scenario: '+text);
}
