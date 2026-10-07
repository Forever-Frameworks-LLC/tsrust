import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertCallableExpressionsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='callable-expressions'||row.id!=='callable_expressions')throw Error('Callable expressions family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Callable expressions fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete callable-expression scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Callable expressions denominator');
 const native=["return (value:int):int=>base+value;", "(c:int):int=>a+b+c;", "return (value:int):int=>value+base;", "while (i < n)", "const direct=make(base)(3);", "const nested=chain(base)(2)(3);", "const inline=((value)=>value*2)(base);", "const named=(function(value:int):int{return value+4;})(base);", "const args=[base,2];const spread=((a,b)=>a+b)(...args);", "const omitted=((value:int=7):int=>value)();", "const trace:int[]=[];const ordered=factory(trace,base)(mark(trace,5));", "trace.push(1);", "trace.push(2);", "output.push([String(direct),String(nested),String(inline),String(named),String(spread),String(omitted),String(ordered),trace.join(\",\"),input+String(base)]);", "return output;"];
 for(let text of native){if(kind==='typescript')text=text.replaceAll(':int',':number').replace('((value)=>','((value:number)=>').replace('((a,b)=>','((a:number,b:number)=>').replace('const args=[base,2];','const args:[number,number]=[base,2];');if(!containsCodeSnippet(source, text))throw Error('Actual callable-expression scenario: '+text);}
}
