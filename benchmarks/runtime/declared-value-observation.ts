// Appended external result observation, never inside timed source workloads.
function observeDeclared(value:unknown,type:string):{checksum:number;exactBits?:string}{
 if(type==='int'){if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw Error('Bounded int scalar required');return{checksum:value};}
 if(type==='boolean'){if(typeof value!=='boolean')throw Error('Boolean scalar required');return{checksum:value?1:0,exactBits:value?'b:1':'b:0'};}
 let checksum=0;const add=(n:number):void=>{checksum=(checksum*131+n)%1000000007;};
 if(type==='number'){if(typeof value!=='number')throw Error('Number scalar required');const data=new DataView(new ArrayBuffer(8));data.setFloat64(0,value,false);const bits=Number.isNaN(value)?0x7ff8000000000000n:data.getBigUint64(0,false);for(let s=56n;s>=0n;s-=8n)add(Number((bits>>s)&255n));return{checksum,exactBits:'n:'+(Number.isNaN(value)?'nan':bits.toString(16).padStart(16,'0'))};}
 function visit(value:unknown,type:string):string{
  if(type.endsWith('[]')){if(!Array.isArray(value))throw Error('Complete array shape required');add(256);const tokens:string[]=[];for(const item of value)tokens.push(visit(item,type.slice(0,-2)));add(257);return'['+tokens.join(',')+']';}
  if(type==='Row'||type==='Node'){
   if(typeof value!=='object'||value===null||Array.isArray(value))throw Error('Record object required');
   const record=value as Record<string,unknown>,tail=type==='Row'?'items':'children';
   if(Object.keys(record).length!==3||!Object.hasOwn(record,'code')||!Object.hasOwn(record,'text')||!Object.hasOwn(record,tail))throw Error('Exact separate Row3/Node3 schema required');
   add(256);const tokens=[visit(record.code,'int'),visit(record.text,'string'),visit(record[tail],type==='Row'?'int[]':'Node[]')];add(257);return'['+tokens.join(',')+']';
  }
  if(type==='int'){if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw Error('Bounded int field required');add(value);return String(value);}
  if(type!=='string'||typeof value!=='string')throw Error('UTF8 string field required');add(258);const codes:number[]=[];for(const scalar of value){const code=scalar.codePointAt(0)!;if(code>=0xd800&&code<=0xdfff)throw Error('Valid Unicode required');add(code);codes.push(code);}add(259);return's('+codes.join(',')+')';
 }
 const shape=visit(value,type);return{checksum,exactBits:'t:'+shape};
}
