import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertFunctionExpressionsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='function-expressions'||row.id!=='function_expressions')throw Error('Function expression family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Function expression fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete function-expression scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Function expression denominator');
 for(const text of ['while (i < n)','const scale=function (value:int):int{return value*3+base;};','tags.map(function (tag){return tag.length;})','words.sort(function (left,right){return left<right?-1:1;});','sizes.forEach(function (size:int){total+=size;});','const join=function (first:string="a",...rest:string[]):string{return first+String(rest.length);};','output.push([String(apply(scale,i%5)),String(total),words[0].slice(),join(),join("b","c","d")]);','return output;'])if(!containsCodeSnippet(source, text))throw Error('Actual function-expression scenario: '+text);
 if(containsCodeSnippet(source, '=>{')||containsCodeSnippet(source, '=> {'))throw Error('Arrow function in a function-expression scenario');
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let expressions=0;
  (function visit(node){if(ts.isFunctionExpression(node)&&!node.name)expressions++;ts.forEachChild(node,visit);})(file);
  if(file.parseDiagnostics.length||expressions!==5)throw Error('Five actual function expressions');
 }
}
