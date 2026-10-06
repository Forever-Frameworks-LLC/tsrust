// Benchmark-only independent integer reference. No Number bitwise operator,
// measured JS method, or pow result is used as its own oracle.
import fs from 'node:fs';
import {rationalBits,tokensObservation} from './number-oracle.mjs';
const domain=JSON.parse(fs.readFileSync(new URL('./number-operators-domains.json',import.meta.url)));
const U32=1n<<32n,MASK=U32-1n,SIGN=1n<<31n;
const names=['and','or','xor','not','shift_left','shift_right','shift_right_unsigned'];
function method(id){const name=id.replace(/^number_/,'');if(!names.includes(name))throw Error('Unknown exact Number operator; power needs independent approximate reference');return name;}
function guard(n,seed){if(!Number.isSafeInteger(n)||n<0||n>domain.maxIterations||!Number.isSafeInteger(seed)||seed<0||seed>domain.maxSeed)throw Error('Bounded exact Number operator domain');}
function uint(integer){return ((integer%U32)+U32)%U32;}
function signed(integer){return integer>=SIGN?integer-U32:integer;}
function apply(name,a,b=0n){
 if(name==='and')return signed(a&b);
 if(name==='or')return signed(a|b);
 if(name==='xor')return signed(a^b);
 if(name==='not')return signed((~a)&MASK);
 const count=b&31n;
 if(name==='shift_left')return signed((a<<count)&MASK);
 if(name==='shift_right')return signed(a)>>count;
 if(name==='shift_right_unsigned')return a>>count;
 throw Error('Unknown exact operator');
}
export function toUint32FromBits(raw){
 if(typeof raw!=='string'||!/^[0-9a-f]{16}$/.test(raw))throw Error('Binary64 input hex required');
 const value=BigInt('0x'+raw),exponent=(value>>52n)&2047n,fraction=value&0xfffffffffffffn;
 if(exponent===2047n)return 0n; // NaN and either infinity map to0.
 const mantissa=exponent===0n?fraction:(1n<<52n)|fraction;
 const power=exponent===0n?-1074:Number(exponent)-1075;
 const magnitude=power>=0?mantissa<<BigInt(power):mantissa>>BigInt(-power);
 return uint((value&(1n<<63n))!==0n?-magnitude:magnitude);
}
export function referenceNumberOperatorBits(id,leftBits,rightBits='0000000000000000'){
 return rationalBits(apply(method(id),toUint32FromBits(leftBits),toUint32FromBits(rightBits)),1n);
}
export function independentOperandQuarters(id,index,seed){
 method(id);guard(index,seed);const i=BigInt(index),s=BigInt(seed),jitter=(i*17n+s)%5n;
 const left=4n*BigInt(domain.leftIntegerTable[Number((i*13n+s)%16n)])+jitter;
 const name=method(id),table=name.startsWith('shift_')?domain.shiftIntegerTable:domain.rightIntegerTable;
 const right=4n*BigInt(table[Number((i*7n+s)%16n)])+(name.startsWith('shift_')?jitter:-jitter);
 return {left,right};
}
export function expectedNumberOperators(id,n,seed){
 const name=method(id);guard(n,seed);let total=BigInt(seed),trace=BigInt(seed)%1009n,calls=0n;
 for(let index=0;index<n;index++){
  const {left,right}=independentOperandQuarters(id,index,seed);
  trace=(trace*17n+1n)%1009n;calls++;
  if(name!=='not'){trace=(trace*17n+2n)%1009n;calls++;}
  total+=apply(name,uint(left/4n),uint(right/4n));
 }
 // All signed/unsigned results and marker values are integers. At n<=500000,
 // every partial sum and final addition has absolute magnitude<2^51. Ordered
 // binary64 additions are exact; this does not assume only the final rounding.
 return tokensObservation([rationalBits(total+trace+calls,1n)]);
}
export function expectedNumberOperatorEffects(){
 const left=rationalBits(-17179869183n,4n),right=rationalBits(135n,4n),tokens=[];
 for(const name of names){tokens.push(referenceNumberOperatorBits('number_'+name,left,right),rationalBits(name==='not'?1n:19n,1n),rationalBits(name==='not'?1n:2n,1n));}
 // Exact special-answer cases are untimed semantic checks only. The power
 // throughput row instead reaches the40 nontrivial Gate49 pow input tuples.
 for(const [value,trace,calls]of [[8n,19n,2n],[512n,326n,3n],[-8n,1n,1n],[1n,19n,2n]])tokens.push(rationalBits(value,1n),rationalBits(trace,1n),rationalBits(calls,1n));
 return tokensObservation(tokens,true);
}
export const directInputCases=[
 ['0000000000000000','3ff0000000000000'],['8000000000000000','bff0000000000000'],
 ['7ff8000000000000','4040000000000000'],['7ff0000000000000','4040800000000000'],['fff0000000000000','bfe8000000000000'],
 ['41e0000000000000','403f000000000000'],['41efffffffe00000','4040000000000000'],
 ['41f0000000000000','bff0000000000000'],['c1f0000000100000','4040800000000000']
];
