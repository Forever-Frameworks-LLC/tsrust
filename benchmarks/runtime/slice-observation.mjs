// Proposed complete-shape observer; appended/adapted outside timed workload.
export function observeSliceTree(value,{numbers=false}={}) {
 let checksum=0;const view=numbers?new DataView(new ArrayBuffer(8)):undefined;
 const add=code=>{checksum=(checksum*131+code)%1000000007;};
 function visit(value){
  if(Array.isArray(value)){add(256);const tokens=value.map(visit);add(257);return '['+tokens.join(',')+']';}
  if(numbers){if(typeof value!=='number')throw new Error('Number leaf required');const nan=Number.isNaN(value);view.setFloat64(0,nan?NaN:value,false);const bits=nan?0x7ff8000000000000n:view.getBigUint64(0,false);for(let shift=56n;shift>=0n;shift-=8n)add(Number((bits>>shift)&255n));return nan?'nan':bits.toString(16).padStart(16,'0');}
  if(typeof value==='boolean'){add(value?261:260);return value?'b1':'b0';}
  if(typeof value==='number'){if(!Number.isSafeInteger(value)||value<0)throw new Error('Bounded integer leaf required');add(value);return String(value);}
  if(typeof value==='string'){add(258);const codes=[];for(const scalar of value){const code=scalar.codePointAt(0);if(code>=0xd800&&code<=0xdfff)throw new Error('Invalid Unicode leaf');add(code);codes.push(code);}add(259);return 's('+codes.join(',')+')';}
  throw new Error('Unexpected array slice leaf');
 }
 const shape=visit(value);return {checksum,exactBits:(numbers?'m:':'t:')+shape};
}
