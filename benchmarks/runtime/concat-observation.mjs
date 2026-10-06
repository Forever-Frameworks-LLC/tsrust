// Benchmark-only declared concat results. Legacy exact observers are unchanged.
const familyResults={int:'int[][]',number:'number[][]',boolean:'boolean[][]',string:'string[][]',record:'Row[][]',nested_int:'int[][][]',nested_number:'number[][][]',nested_boolean:'boolean[][][]',nested_string:'string[][][]',nested_record:'Row[][][]',deep_int:'int[][][][]',recursive_record:'Node[][]'};
export const concatFields={Row:[['code','int'],['text','string'],['items','int[]']],Node:[['code','int'],['text','string'],['children','Node[]']]};
export function isConcat(w){return w?.id?.startsWith('array_concat_')||w?.concatObserver!==undefined;}
export function assertConcatPolicy(w){
 const scalar=w?.id==='array_concat_ordered_trace',m=/^array_concat_(.+)_(zero|variadic)$/.exec(w?.id??'');
 if(w?.concatObserver!=='declared-concat-v1'||w.concatMaxDepth!==16||w.approximateValidation!==undefined)throw Error('Unknown concat result policy');
 if(scalar){if(w.result!=='int'||w.resultEncoding!==undefined||w.inputText!==undefined||w.recordSchema!==undefined)throw Error('Invalid concat scalar result');return;}
 if(!m||!Object.hasOwn(familyResults,m[1])||w.concatDomain!==m[1]||w.concatMode!==m[2]||w.result!==familyResults[m[1]])throw Error('Unknown concat declared result');
 const encoding=m[1]==='number'||m[1]==='nested_number'?'ieee-f64-tree-v1':'array-shape-scalar-v1';if(w.resultEncoding!==encoding)throw Error('Unknown concat result encoding');
 const record=w.result.includes('Row')?'Row':w.result.includes('Node')?'Node':null;
 const expected=record?{record,fields:concatFields[record].map(([name,type])=>({name,type}))}:undefined;
 if(JSON.stringify(w.recordSchema)!==JSON.stringify(expected))throw Error('Concat nominal schema changed');
 const input=['string','record','nested_string','nested_record','recursive_record'].includes(m[1]);if(input!==(typeof w.inputText==='string'))throw Error('Concat input ABI changed');
}
export function validateConcatObservation(actual,expected,w){
 assertConcatPolicy(w);if(w.result==='int'){if(!Number.isSafeInteger(actual?.checksum)||actual.checksum!==expected?.checksum)throw Error('Concat scalar disagrees with oracle');return;}
 const prefix=w.resultEncoding==='ieee-f64-tree-v1'?'m:':'t:';
 function decode(text){
  if(typeof text!=='string'||!text.startsWith(prefix)||text.length>16777216)throw Error('Invalid complete concat result');let position=2,checksum=0;
  const fail=()=>{throw Error('Invalid complete concat result');},add=code=>{checksum=(checksum*131+code)%1000000007;};
  const take=s=>{if(!text.startsWith(s,position))fail();position+=s.length;};
  function integer(){const m=/^(0|[1-9][0-9]*)/.exec(text.substring(position));if(!m)fail();position+=m[0].length;const value=Number(m[0]);if(!Number.isSafeInteger(value))fail();return value;}
  function visit(type,depth){
   if(depth>16)fail();
   if(type.endsWith('[]')){if(depth>=16)fail();take('[');add(256);if(text[position]!==']'){visit(type.substring(0,type.length-2),depth+1);while(text[position]===','){position++;visit(type.substring(0,type.length-2),depth+1);}}take(']');add(257);return;}
   if(Object.hasOwn(concatFields,type)){if(depth>=16)fail();take('[');add(256);const fields=concatFields[type];for(let i=0;i<fields.length;i++){if(i)take(',');visit(fields[i][1],depth+1);}take(']');add(257);return;}
   if(type==='string'){take('s(');add(258);if(text[position]!==')'){while(true){const code=integer();if(code>0x10ffff||code>=0xd800&&code<=0xdfff)fail();add(code);if(text[position]!==',')break;position++;}}take(')');add(259);return;}
   if(type==='number'){const m=/^(nan|[0-9a-f]{16})/.exec(text.substring(position));if(!m)fail();position+=m[0].length;const bits=m[0]==='nan'?0x7ff8000000000000n:BigInt('0x'+m[0]);if(m[0]!=='nan'&&(bits&0x7ff0000000000000n)===0x7ff0000000000000n&&(bits&0xfffffffffffffn)!==0n)fail();for(let shift=56n;shift>=0n;shift-=8n)add(Number(bits>>shift&255n));return;}
   if(type!=='int'&&type!=='boolean')fail();const value=integer();if(type==='boolean'&&value>1)fail();add(value);
  }
  visit(w.result,0);if(position!==text.length)fail();return checksum;
 }
 if(!Number.isSafeInteger(actual?.checksum)||actual.checksum!==decode(actual.exactBits)||!Number.isSafeInteger(expected?.checksum)||expected.checksum!==decode(expected.exactBits)||actual.checksum!==expected.checksum||actual.exactBits!==expected.exactBits)throw Error('Complete concat result disagrees with independent oracle');
}
