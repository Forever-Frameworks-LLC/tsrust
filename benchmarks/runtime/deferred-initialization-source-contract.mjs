import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertDeferredInitializationSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='deferred-initialization'||row.id!=='deferred_initialization')throw Error('Deferred initialization family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Deferred initialization fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete deferred-initialization scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Deferred initialization denominator');
 for(const text of ['while (i < n)','let label:string;','if(base>=64){label="high";}else if(base>=32){label="mid";}else{label="low";}','let size:int,step:int=base%5;','size=step*2+1;','let kind:string;','switch(step){case 0:kind="zero";break;case 1:case 2:kind="few";break;default:kind="many";}','let text:string;','{const prefix=String(base);text=prefix+input;}','let note:string|undefined;','const absent=note===undefined;','output.push([label,String(size),kind,text,String(absent)]);','return output;'])if(!source.includes(text))throw Error('Actual deferred-initialization scenario: '+text);
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);const deferred=[];
  (function visit(node){if(ts.isVariableDeclaration(node)&&!node.initializer&&ts.isIdentifier(node.name)&&!ts.isForOfStatement(node.parent.parent))deferred.push(node.name.text);ts.forEachChild(node,visit);})(file);
  if(file.parseDiagnostics.length||deferred.join()!=='label,size,kind,text,note')throw Error('Five declarations without initializers: '+deferred.join());
 }
}
