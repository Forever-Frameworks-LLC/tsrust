import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertExceptionCompletionSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='exception_completion'||row.family!=='exception-completion')throw Error('Exception family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Exception fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete exception completion scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Exception denominator');
 for(const text of ['if(value%3===0){throw value;}','return value+1;','return error+2;','finally{trace.push(value);}','finally{return value+4;}','const result=attempt(value,trace);','trace.join(",")','const owned=retained(input+String(value),value%2===0,trace);','if(early){return value;}','finally{trace.push(7);}','const field=retainedField({text:owned,stamp:11},value%2===0,trace);','if(early){return value.text;}','finally{trace.push(value.stamp);}','const mapped=retainedMap({text:field,stamp:1},value%2===0,trace);','if(early){return value.text;}handlers.clear();','handlers.clear();','handlers.set("old",{run:():'+(kind==='native'?'int':'number')+'=>old[0]});','handlers.set("new",{run:():'+(kind==='native'?'int':'number')+'=>current[0]});','const item=handlers.get(early?"old":"new");','trace.push(item.run()+value.stamp);','const looped=retainedLoop({text:mapped,stamp:0},value%4,trace);','const caught=retainedCatch({text:looped,stamp:0},value%3,trace);','const selected=conditionalCleanup(caught,value%2===0,value%3===0);','trace.join(","),selected','return output;'])if(!containsCodeSnippet(source, text))throw Error('Exception scenario: '+text);
 const integer=kind==='native'?'int':'number';
 const fieldFunction='function retainedField(value:RetainedOwner,early:boolean,trace:'+integer+'[]):string{try{if(early){return value.text;}}finally{trace.push(value.stamp);}return value.text;}';
 if(!containsCodeSnippet(source,fieldFunction))throw Error('Exception retained-field control flow');
 const loopFunction='function retainedLoop(value:RetainedOwner,mode:'+integer+',trace:'+integer+'[]):string{for(let i=0;i<2;i+=1){try{if(mode===0){return value.text;}if(mode===1){break;}if(mode===2){continue;}}finally{trace.push(20+i);}}return value.text;}';
 if(!containsCodeSnippet(source,loopFunction))throw Error('Exception loop field completion');
 const catchFunction='function retainedCatch(value:RetainedOwner,mode:'+integer+',trace:'+integer+'[]):string{try{if(mode===0){return value.text;}if(mode===1){throw 31;}trace.push(29);}catch(error){trace.push(30);if(mode===1){return value.text;}}finally{trace.push(31);}return value.text;}';
 if(!containsCodeSnippet(source,catchFunction))throw Error('Exception caught field completion');
 const cleanupFunction='function conditionalCleanup(value:string,early:boolean,replace:boolean):string{const pair:CleanupChoice={first:value,second:"alternate"};try{if(early){return pair.first;}}finally{if(replace){return pair.second;}}return pair.second;}';
 if(!containsCodeSnippet(source,cleanupFunction))throw Error('Exception conditional cleanup field');

}
