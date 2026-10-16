// Pinned declaration inventory, not an assertion of semantic or performance parity.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const require=createRequire(import.meta.url);
const ts=require(path.join(root,'benchmarks/runtime/node_modules/typescript'));
assert.equal(ts.version,'5.9.3');
const libraries=[['date-fns','4.1.0','benchmarks/calendar/node_modules/date-fns','index.d.ts'],['decimal.js','10.6.0','benchmarks/decimal/node_modules/decimal.js','decimal.d.ts']];
const calendarNames=new Set(['addDays','addMonths','differenceInCalendarDays','compareAsc','isLeapYear','getYear','getMonth','getDate','startOfMonth','endOfMonth','format','parseISO','formatISO']);
const decimalNames=new Set(['add','plus','sub','minus','mul','times','div','dividedBy','cmp','comparedTo','toFixed','toNumber']);
const result=[];
for(const [name,version,directory,entry] of libraries){
 const folder=path.join(root,directory);assert.equal(JSON.parse(fs.readFileSync(path.join(folder,'package.json'))).version,version);
 const file=path.join(folder,entry);const program=ts.createProgram([file],{strict:true,noEmit:true,target:ts.ScriptTarget.ES2022,skipLibCheck:true});const checker=program.getTypeChecker();
 const source=program.getSourceFile(file);const symbol=checker.getSymbolAtLocation(source);assert.ok(symbol);
 const exports=checker.getExportsOfModule(symbol);const rows=[];
 const describe=(symbol,scope)=>{
  const resolved=symbol.flags&ts.SymbolFlags.Alias?checker.getAliasedSymbol(symbol):symbol;
  const declaration=resolved.valueDeclaration??resolved.declarations?.[0]??source;
  const type=checker.getTypeOfSymbolAtLocation(resolved,declaration);
  const selected=(name==='date-fns'?calendarNames:decimalNames).has(symbol.name);
  rows.push({name:symbol.name,scope,classification:selected?'partial: named counterpart with checked ranges and distinct error/precision rules':'deferred: no required application use established',signatures:checker.getSignaturesOfType(type,ts.SignatureKind.Call).map(signature=>checker.signatureToString(signature)),type:checker.typeToString(type)});
 };
 for(const symbol of exports){describe(symbol,'export');}
 if(name==='decimal.js'){
  const symbol=exports.find(symbol=>symbol.name==='Decimal');assert.ok(symbol);
  const resolved=symbol.flags&ts.SymbolFlags.Alias?checker.getAliasedSymbol(symbol):symbol;
  for(const member of checker.getPropertiesOfType(checker.getDeclaredTypeOfSymbol(resolved)))describe(member,'instance');
 }
 assert.ok(rows.length>20);result.push({name,version,rows});
}
assert.ok(result[0].rows.find(row=>row.name==='addMonths')?.signatures.length);
assert.ok(result[1].rows.find(row=>row.name==='toFixed'&&row.scope==='instance')?.signatures.length);
console.log(JSON.stringify({typescript:ts.version,status:'declaration discovery and scoped classification; native acceptance is separate',libraries:result},null,2));
