import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertTypeAliasesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='type_aliases'||row.family!=='type-aliases')throw Error('Type alias family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Type alias fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete type alias scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Type alias denominator');
 for(const text of ['type Id = int | string;','type Ids = Id[];','type Count = int;','type Pair = [Count, string];','type Row = string[];','type Rows = Row[];','function kind(id: Id): string','const output:Rows=[];','const ids:Ids=[k,String(input)];','const pair:Pair=[k*2,String(k)];','const [doubled,label]=pair;','const row:Row=[kind(ids[0]),kind(ids[1]),String(doubled),String(label),String(ids.length)];','return output;'])if(!source.includes(text))throw Error('Type alias scenario: '+text);
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);const names=[];
  (function visit(node){if(ts.isTypeAliasDeclaration(node))names.push(node.name.text);ts.forEachChild(node,visit);})(file);
  if(file.parseDiagnostics.length||names.join()!=='int,Id,Ids,Count,Pair,Row,Rows')throw Error('Type alias declarations: '+names.join());
 }
}
