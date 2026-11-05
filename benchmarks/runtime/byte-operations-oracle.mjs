import {observeSliceTree} from './slice-observation.mjs';
import {byteOperationCases} from './byte-operations-workloads.mjs';
// Independent mathematical/bit oracles; does not call Buffer's integer codec APIs.
export function integerBytes(value,width,order,signed) {
  if(![1,2,4,...(signed?[8]:[])].includes(width)) throw new RangeError('width');
  value=BigInt(value); const bits=BigInt(width*8), modulus=1n<<bits;
  const minimum=signed?-(modulus>>1n):0n, maximum=signed?(modulus>>1n)-1n:modulus-1n;
  if(value<minimum||value>maximum) throw new RangeError('value');
  if(order!=='little'&&order!=='big') throw new RangeError('byte-order');
  if(value<0n)value+=modulus;
  const bytes=Array.from({length:width},()=>{const byte=Number(value&255n);value>>=8n;return byte;});
  return order==='little'?bytes:bytes.reverse();
}
export function integerValue(bytes,order,signed) {
  if(order!=='little'&&order!=='big')throw new RangeError('byte-order');
  const ordered=order==='little'?[...bytes].reverse():bytes;
  let value=0n;for(const byte of ordered)value=value*256n+BigInt(byte);
  const bits=BigInt(bytes.length*8); if(signed&&value>=(1n<<(bits-1n)))value-=1n<<bits;
  return value;
}
export function byteSequenceIndex(haystack,needle,start) {
  if(!Number.isSafeInteger(start)||start<0||start>haystack.length)throw new RangeError('range');
  if(needle.length===0)return start;
  outer:for(let index=start;index+needle.length<=haystack.length;index++){
    for(let offset=0;offset<needle.length;offset++)if(haystack[index+offset]!==needle[offset])continue outer;
    return index;
  }
  return -1;
}
export function copiedWithin(bytes,target,start,end) {
  if(![target,start,end].every(Number.isSafeInteger)||target<0||start<0||end<start||end>bytes.length||target>bytes.length-(end-start))throw new RangeError('range');
  const result=Array.from(bytes), original=Array.from(bytes);
  for(let i=0;i<end-start;i++)result[target+i]=original[start+i];
  return result;
}
export const packetVectors=[
  {value:'-9223372036854775808',width:8,order:'little',signed:true,bytes:[0,0,0,0,0,0,0,128]},
  {value:'9223372036854775807',width:8,order:'big',signed:true,bytes:[127,255,255,255,255,255,255,255]},
  {value:'-32768',width:2,order:'big',signed:true,bytes:[128,0]},
  {value:'4294967295',width:4,order:'little',signed:false,bytes:[255,255,255,255]},
  {value:'255',width:1,order:'little',signed:false,bytes:[255]},
];
export const floatVectors=[
  {width:4,order:'little',value:-0,bytes:[0,0,0,128]},
  {width:4,order:'big',value:Infinity,bytes:[127,128,0,0]},
  {width:4,order:'little',value:1.5,bytes:[0,0,192,63]},
  {width:4,order:'big',value:1+2**-24,bytes:[63,128,0,0]},
  {width:8,order:'little',value:-0,bytes:[0,0,0,0,0,0,0,128]},
  {width:8,order:'big',value:1.5,bytes:[63,248,0,0,0,0,0,0]},
];

const packetIntegers=[[65,1,true],[-12345,2,true],[-1234567,4,true],[123456789,8,true],[255,1,false],[65535,2,false],[4294967295,4,false]];
const packetByteChecksum=packetIntegers.flatMap(([value,width,signed])=>integerBytes(BigInt(value),width,'little',signed)).reduce((sum,byte)=>sum+byte,0)+255+194;
const packetNumericChecksum=packetIntegers.reduce((sum,[value])=>sum+value*4,0)-3;
export function expectedByteOperationValues(id,n,seed,input){
 const row=byteOperationCases.find(row=>row.id===id&&row.inputText===input);
 if(!row||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||seed<0)throw Error('Byte operation identity/input');
 const size=row.bytes,operation=row.operation;let values;
 if(operation==='concat')values=[size*2,97,98];
 else if(operation==='copy_into')values=[size+1,0,97];
 else if(operation.startsWith('copy_within'))values=[size,97,97];
 else if(operation==='equals_equal')values=[1,size,size];
 else if(operation==='equals_tail')values=[0,size,size];
 else if(operation==='compare_equal')values=[1,size,size];
 else if(operation==='compare_tail')values=[0,size,size];
 else if(operation==='index_tail')values=[size-3,size,4];
 else if(operation==='index_miss')values=[0,size,4];
 else if(operation==='index_repetitive')values=[0,size,Math.min(size,512)/2];
 else if(operation==='packet_read')values=[size,packetNumericChecksum*(size/64),size/64];
 else if(operation==='packet_write')values=[size,packetByteChecksum*(size/64),size/64];
 else throw Error('Byte operation');
 return Array.from({length:n},()=>values).flat();
}
export function expectedByteOperationObservation(id,n,seed,input){return observeSliceTree(expectedByteOperationValues(id,n,seed,input));}
