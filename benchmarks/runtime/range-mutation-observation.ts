// Benchmark-only COMPLETE observation outside timing, from declared source schema.
// Actual emitted native owner/types/fields remain pending emission/compile guard.
function observeRange(value: unknown, kind: string): {checksum:number;exactBits:string} {
 let checksum=0;const view=new DataView(new ArrayBuffer(8));
 const add=(code:number):void=>{checksum=(checksum*131+code)%1000000007;};
 type Schema=string|{array:Schema};
 function schema(kind:string):Schema {
  if(!['int','number','boolean','string','record','nested_int','nested_number','nested_boolean','nested_string','nested_record','deep_int','recursive_record'].includes(kind))throw Error('Unknown concat schema');
  if(kind==='deep_int')return{array:{array:'int'}};
  if(kind.startsWith('nested_'))return{array:schema(kind.substring(7))};
  if(kind==='record')return'Row';if(kind==='recursive_record')return'Node';
  if(['int','number','boolean','string'].includes(kind))return kind;
  throw Error('Unknown declared concat schema');
 }
 function visit(value:unknown,shape:Schema,depth:number):string {
  if(depth>16)throw Error('Concat result exceeds declared depth16');
  if(typeof shape==='object'){
   if(depth>=16)throw Error('Concat result exceeds declared depth16');
   if(!Array.isArray(value))throw Error('Array shape required');add(256);const tokens:string[]=[];
   for(const item of value)tokens.push(visit(item,shape.array,depth+1));add(257);return'['+tokens.join(',')+']';
  }
  if(shape==='Row'||shape==='Node'){
   if(depth>=16)throw Error('Concat result exceeds declared depth16');
   if(typeof value!=='object'||value===null||Array.isArray(value))throw Error('Record shape required');
   const record=value as Record<string,unknown>,tail=shape==='Row'?'items':'children';
   if(Object.keys(record).length!==3||!Object.hasOwn(record,'code')||!Object.hasOwn(record,'text')||!Object.hasOwn(record,tail))throw Error('Declared record fields required');
   add(256);const tokens=[visit(record.code,'int',depth+1),visit(record.text,'string',depth+1),visit(record[tail],{array:shape==='Row'?'int':'Node'},depth+1)];add(257);return'['+tokens.join(',')+']';
  }
  if(shape==='number'){
   if(typeof value!=='number')throw Error('Number leaf required');const nan=Number.isNaN(value);view.setFloat64(0,value,false);const bits=nan?0x7ff8000000000000n:view.getBigUint64(0,false);
   for(let shift=56n;shift>=0n;shift-=8n)add(Number((bits>>shift)&255n));return nan?'nan':bits.toString(16).padStart(16,'0');
  }
  if(shape==='boolean'){if(typeof value!=='boolean')throw Error('Boolean leaf required');const code=value?1:0;add(code);return String(code);}
  if(shape==='int'){if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw Error('Bounded nonnegative int required');add(value);return String(value);}
  if(typeof value!=='string')throw Error('String leaf required');add(258);const codes:number[]=[];
  for(const scalar of value){const code=scalar.codePointAt(0)!;if(code>=0xd800&&code<=0xdfff)throw Error('Invalid Unicode');add(code);codes.push(code);}add(259);return's('+codes.join(',')+')';
 }
 const shape=visit(value,{array:schema(kind)},0);return{checksum,exactBits:(kind==='number'||kind==='nested_number'?'m:':'t:')+shape};
}
