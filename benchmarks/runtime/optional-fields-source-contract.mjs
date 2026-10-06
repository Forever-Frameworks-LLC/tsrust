import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertOptionalFieldsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='optional_fields'||row.family!=='optional-fields')throw Error('Optional field family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Optional field fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete optional field scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Optional field denominator');
 for(const text of ['type Point = { x: int; y?: int; };','return point.x + (point.y ?? 0);','const flat:Point={x:k};','const full:Point={x:k,y:k%7};','let moving:Point={x:1};','moving.y=k%3;','const {y:taken=9}=flat;','output.push([String(width(flat)),String(width(full)),String(width(moving)),String(taken),String(input)]);','return output;'])if(!source.includes(text))throw Error('Optional field scenario: '+text);
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);const optional=[];
  (function visit(node){if(ts.isPropertySignature(node)&&node.questionToken)optional.push(node.name.getText(file));ts.forEachChild(node,visit);})(file);
  if(file.parseDiagnostics.length||optional.join()!=='y')throw Error('Optional property signatures: '+optional.join());
 }
}
