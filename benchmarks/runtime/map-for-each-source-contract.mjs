import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertMapForEachSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='map_for_each'||row.family!=='map-for-each')throw Error('Map callback family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Map callback fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete map callback scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Map callback denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of [`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,`new Map<${integer},${integer}>([[i+seed,2],[i+seed+1,3],[i+seed,4]])`,'values.forEach((value,key,source)=>','total=total+value+key+source.size','new Map<string,string>([[String(input),"first"],["tail","last"],[String(input),"updated"]])','names.forEach((value,key,source)=>','source.has(key)','text=text+key+":"+value+"|"','return output;'])if(!containsCodeSnippet(source, text))throw Error('Map callback scenario: '+text);
}
