// Name matches are candidates, not proof of runtime or ownership equivalence.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const files=process.argv.slice(2);
assert.equal(files.length,3,'Usage: node tools/compare-type-inventories.mjs TYPESCRIPT.json RUST.json TSRUST.json');
const [typescript,rust,tsrust]=files.map(file=>JSON.parse(readFileSync(file,'utf8')));
const mappings={
  'TypeScript language service':{
    boolean:['boolean'], 'int reference':['int'],number:['number'],bigint:[null,'deferred','Arbitrary precision integer syntax is outside the bounded width-adapter scope.'],
    string:['string'],Date:[null,'missing','Calendar/time support is required.'],Uint8Array:['std:bytes'],Array:['Array'],tuple:[null,'intentionally different','Fixed source tuples do not inherit JavaScript array mutation; compare indexed fields with tuple witnesses.'],Map:['Map'],Set:['Set'],
    Promise:[null,'intentionally different','Task/Job use explicit ownership, start and settlement contracts.'],JSON:['JSON'],
    null:[null,'intentionally different','Null is a primitive union alternative, not an object receiver.'],undefined:[null,'intentionally different','Undefined is a primitive union alternative, not an object receiver.'],
  },
  'rust-analyzer':{
    i64:['int'],f64:['number'],bool:['boolean'],String:['string'],'Vec<u8>':['std:bytes'],
    'Option<i64>':[null,'intentionally different','Optional unions and match/narrowing replace the Rust Option receiver API.'],
    'Result<i64,String>':[null,'intentionally different','Source Result and checked matching have their own payload/ownership contract.'],
    Duration:[null,'missing','A first-class checked duration contract is required.'],SystemTime:[null,'missing','UTC/calendar time support is required.'],
    'HashMap<String,i64>':['Map'],
  },
};
for(const type of ['i8','i16','i32','u8','u16','u32','u64','f32'])mappings['rust-analyzer'][type]=[null,'missing','Checked width/precision boundary mapping is required; int/number are not a lossless blanket substitute.'];
for(const type of ['i128','u128'])mappings['rust-analyzer'][type]=[null,'deferred','Full 128-bit arithmetic is outside this bounded type-system increment; require an explicit checked textual/domain adapter.'];
const rows=[];
for(const inventory of [typescript,rust]){
  for(const type of inventory.rows){
    const mapping=mappings[inventory.engine][type.type];assert.ok(mapping,`Classify new reference type ${type.type}`);
    const [group,disposition,reason]=mapping;
    const catalog=tsrust.rows.filter(row=>row.group===group && row.catalog);
    const classify=member=>{
      const bare=member.name.replace(/\(.*$/,'');
      const camel=bare.replace(/_([a-z])/g,(_,letter)=>letter.toUpperCase());
      const counterpart=catalog.find(row=>row.name===bare || row.name===camel);
      return {name:member.name,signature:member.signature,classification:disposition ?? (counterpart?'partial':'deferred'),
        reason:reason ?? (counterpart?'Catalog counterpart found; overload, runtime and ownership equivalence require the linked behavioral witnesses.':'No receiver-name counterpart; prioritize concrete UI/service needs rather than universal API compatibility.'),
        counterpart:counterpart?{identity:counterpart.identity,signature:counterpart.catalog.signature,examples:counterpart.catalog.examples ?? []}:null};
    };
    rows.push({engine:inventory.engine,version:inventory.version,type:type.type,sourceGroup:group,
      classification:disposition ?? 'partial',members:type.members.map(classify)});
  }
}
assert.ok(rows.some(row=>row.type==='Date' && row.classification==='missing'));
assert.ok(rows.some(row=>row.type==='Array' && row.members.some(member=>member.counterpart?.identity==='Array::push')));
const counts={};for(const row of rows)for(const member of row.members)counts[member.classification]=(counts[member.classification]??0)+1;
console.log(JSON.stringify({schemaVersion:1,status:'initial comparison; partial means declaration candidate, not behavioral acceptance',counts,rows,constructorsAndSignatures:[typescript.signatures,rust.signatures]},null,2));
