import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertInterfacesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='interfaces'||row.family!=='interfaces')throw Error('Interface family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Interface fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete interface scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Interface denominator');
 for(const text of ['interface Named { name: string; }','interface Positioned { x: int; y?: int }','interface Player extends Named, Positioned { score: int, }','function describe(player: Player): string','const plain:Player={name:String(input),x:k,score:k%5};','const placed:Player={name:"p",x:k,y:k%3,score:1};','const named:Named={name:String(k)};','output.push([describe(plain),describe(placed),String(named.name),String(plain.score+placed.score),String(placed.y??-1)]);','return output;'])if(!source.includes(text))throw Error('Interface scenario: '+text);
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);const found=[];
  (function visit(node){if(ts.isInterfaceDeclaration(node))found.push(node.name.text+':'+(node.heritageClauses??[]).flatMap(clause=>clause.types.map(type=>type.expression.getText(file))).join('+'));ts.forEachChild(node,visit);})(file);
  if(file.parseDiagnostics.length||found.join()!=='Named:,Positioned:,Player:Named+Positioned')throw Error('Interface declarations: '+found.join());
 }
}
