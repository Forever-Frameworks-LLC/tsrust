// Benchmark-only complete observation. No workload output is flattened in timing.
const reductionFields: [string,string][]=[['total','int'],['text','string'],['rows','int[][]'],['trace','int'],['visits','int']];
function observeReduction(value:unknown,type:string): {checksum:number;exactBits:string} {
 let checksum=0;
 const add=(code:number):void=>{checksum=(checksum*131+code)%1000000007;};
 function visit(value:unknown,type:string,depth:number):string {
  if(depth>8)throw Error('Reduction result exceeds declared rank');
  if(type.endsWith('[]')){if(!Array.isArray(value))throw Error('Reduction array required');add(256);const parts=value.map(item=>visit(item,type.slice(0,-2),depth+1));add(257);return 'a('+parts.join(',')+')';}
  if(type==='int'){if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw Error('Bounded reduction integer required');add(value);return 'i('+value+')';}
  if(type==='boolean'){if(typeof value!=='boolean')throw Error('Reduction Boolean required');add(262);add(value?1:0);return 'b('+(value?'1':'0')+')';}
  if(type==='string'){if(typeof value!=='string')throw Error('Reduction String required');add(258);const codes:number[]=[];for(const scalar of value){const code=scalar.codePointAt(0)!;if(code>=0xd800&&code<=0xdfff)throw Error('Invalid Unicode reduction result');add(code);codes.push(code);}add(259);return 's('+codes.join(',')+')';}
  if(type==='Row'){if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==reductionFields.length||reductionFields.some(([key])=>!Object.hasOwn(value,key)))throw Error('Complete declared reduction Row required');add(260);const parts=reductionFields.map(([key,fieldType])=>visit((value as Record<string,unknown>)[key],fieldType,depth+1));add(261);return 'o('+parts.join(',')+')';}
  throw Error('Unsupported reduction observation type');
 }
 const shape=visit(value,type,0);return {checksum,exactBits:'r:'+shape};
}
