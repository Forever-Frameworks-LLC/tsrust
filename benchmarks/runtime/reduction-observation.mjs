// Benchmark-only complete observation. No workload output is flattened in timing.
export const reductionFields=[['total','int'],['text','string'],['rows','int[][]'],['trace','int'],['visits','int']];
export function isReduction(workload){return workload.reductionObserver==='declared-result-v1';}
export function isReductionSchema(workload){return isReduction(workload)&&workload.resultEncoding==='reduction-schema-v1';}
export function observeReduction(value,type){
 let checksum=0;
 const add=code=>{checksum=(checksum*131+code)%1000000007;};
 function visit(value,type,depth){
  if(depth>8)throw Error('Reduction result exceeds declared rank');
  if(type.endsWith('[]')){if(!Array.isArray(value))throw Error('Reduction array required');add(256);const parts=value.map(item=>visit(item,type.slice(0,-2),depth+1));add(257);return 'a('+parts.join(',')+')';}
  if(type==='int'){if(!Number.isSafeInteger(value)||value<0)throw Error('Bounded reduction integer required');add(value);return 'i('+value+')';}
  if(type==='boolean'){if(typeof value!=='boolean')throw Error('Reduction Boolean required');add(262);add(value?1:0);return 'b('+(value?'1':'0')+')';}
  if(type==='string'){if(typeof value!=='string')throw Error('Reduction String required');add(258);const codes=[];for(const scalar of value){const code=scalar.codePointAt(0);if(code>=0xd800&&code<=0xdfff)throw Error('Invalid Unicode reduction result');add(code);codes.push(code);}add(259);return 's('+codes.join(',')+')';}
  if(type==='Row'){if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==reductionFields.length||reductionFields.some(([key])=>!Object.hasOwn(value,key)))throw Error('Complete declared reduction Row required');add(260);const parts=reductionFields.map(([key,fieldType])=>visit(value[key],fieldType,depth+1));add(261);return 'o('+parts.join(',')+')';}
  throw Error('Unsupported reduction observation type');
 }
 const shape=visit(value,type,0);return {checksum,exactBits:'r:'+shape};
}
// Decode the self-typed external token and then re-observe it canonically.
// Record field order is the declared Row schema; missing/extra fields are rejected.
export function validateReductionObservation(actual,expected){
 if(!Number.isSafeInteger(actual?.checksum)||typeof actual?.exactBits!=='string'||!actual.exactBits.startsWith('r:'))throw Error('Invalid exact reduction observation');
 const text=actual.exactBits;let pos=2;
 const fail=()=>{throw Error('Invalid exact reduction observation');};
 const take=s=>{if(!text.startsWith(s,pos))fail();pos+=s.length;};
 function integer(){const match=/^(0|[1-9][0-9]*)/.exec(text.slice(pos));if(!match)fail();pos+=match[0].length;const n=Number(match[0]);if(!Number.isSafeInteger(n))fail();return n;}
 function visit(depth){
  if(depth>8)fail();const kind=text[pos++];take('(');
  if(kind==='i'){const value=integer();take(')');return {value,type:'int'};}
  if(kind==='b'){const n=integer();if(n>1)fail();take(')');return {value:n===1,type:'boolean'};}
  if(kind==='s'){let value='';if(text[pos]!==')'){while(true){const code=integer();if(code>0x10ffff||code>=0xd800&&code<=0xdfff)fail();value+=String.fromCodePoint(code);if(text[pos]!==',')break;pos++;}}take(')');return {value,type:'string'};}
  if(kind==='a'||kind==='o'){
   const values=[];if(text[pos]!==')'){while(true){values.push(visit(depth+1));if(text[pos]!==',')break;pos++;}}take(')');
   if(kind==='o'){if(values.length!==reductionFields.length||values[0].type!=='int'||values[1].type!=='string'||values[2].type!=='array'||values[3].type!=='int'||values[4].type!=='int'||values[2].value.some(row=>!Array.isArray(row)||row.some(n=>!Number.isSafeInteger(n)||n<0)))fail();return {value:Object.fromEntries(values.map((v,i)=>[reductionFields[i][0],v.value])),type:'Row'};}
   return {value:values.map(v=>v.value),type:'array'};
  }
  fail();
 }
 const decoded=visit(0);if(pos!==text.length)fail();
 // Exact expected bytes bind the complete schema and all empty-rank delimiters.
 // Recompute the checksum from the decoded token, never trusting a supplied sum.
 let checksum=0;const add=code=>{checksum=(checksum*131+code)%1000000007;};
 function check(v){if(Array.isArray(v)){add(256);for(const item of v)check(item);add(257);}else if(typeof v==='boolean'){add(262);add(v?1:0);}else if(typeof v==='number')add(v);else if(typeof v==='string'){add(258);for(const scalar of v)add(scalar.codePointAt(0));add(259);}else{add(260);for(const [key]of reductionFields)check(v[key]);add(261);}}
 check(decoded.value);
 if(actual.checksum!==checksum||actual.checksum!==expected?.checksum||actual.exactBits!==expected?.exactBits)throw Error('Exact reduction result disagrees with independent oracle');
}
