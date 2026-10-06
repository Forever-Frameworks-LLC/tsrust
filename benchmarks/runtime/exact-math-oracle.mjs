import {rationalBits,tokensObservation} from './number-oracle.mjs';
const U32=1n<<32n;
function guard(id,n,seed){if(!/^math_(round|sign|fround|min|max|clz32|imul)$/.test(id))throw new Error('Unknown exactMath workload');if(!Number.isSafeInteger(n)||n<0||n>500000||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw new Error('Bounded exactMath domain');}
function floorDivide(n,d){return n>=0n?n/d:-((-n+d-1n)/d);}
function nearestEven(n,d){const q=n/d,r=n%d;return q+(2n*r>d||2n*r===d&&(q&1n)!==0n?1n:0n);}
function uint32Quarter(q){const integer=q/4n;return ((integer%U32)+U32)%U32;}
function leadingZeros(bits){let result=0;for(let flag=1n<<31n;flag>0n;flag>>=1n){if((bits&flag)!==0n)break;result++;}return BigInt(result);}
// The timed fround domain crosses binary32 spacing1/2 around2^24.
// Exact quarter rational input; independent midpoint/even quantization.
function froundQuarter(q){const spacing=q<4n*(1n<<24n)?4n:8n;return nearestEven(q,spacing)*(spacing/4n);}
export function expectedExactMath(id,n,seed){guard(id,n,seed);const s=BigInt(seed);let eighths=8n*s;for(let k=0;k<n;k++){const i=BigInt(k);let result;
 if(id==='math_round'){const q=(i*13n+s)%65n-32n,lower=floorDivide(q,4n);result=8n*(lower+(q-lower*4n>=2n?1n:0n));}
 else if(id==='math_sign'){const q=(i*17n+s)%65n-32n;result=q<0n?-8n:q>0n?8n:0n;}
 else if(id==='math_fround'){const q=4n*(1n<<24n)+((i*13n+s)%2049n-1024n);result=8n*froundQuarter(q);}
 else if(id==='math_min'||id==='math_max'){const values=[(i*13n+s)%65n-32n,2n*((i*17n+s)%81n-40n),4n*((i*19n+s)%97n-48n)];result=values[0];for(let j=1;j<3;j++)if(id==='math_min'?values[j]<result:values[j]>result)result=values[j];}
 else if(id==='math_clz32'){const q=4n*8192n*((i*104729n+s)%1048576n-524288n)+i%4n;result=8n*leadingZeros(uint32Quarter(q));}
 else{const a=4n*8192n*((i*13n+s)%1048576n-524288n)+3n,b=4n*65536n*((i*17n+s)%65537n-32768n)-2n;let product=uint32Quarter(a)*uint32Quarter(b)%U32;if(product>=1n<<31n)product-=U32;result=8n*product;}
 eighths+=result;}
 // Every input/result is an exact eighth or integer. Bounded sums stay below2^53,
 // so ordered binary64 additions are exact, rather than assumed final rounding.
 return tokensObservation([rationalBits(eighths,8n)]);
}
export const directSemanticCases={
 round:[{input:'-0',expected:'8000000000000000'},{input:'-0.5',expected:'8000000000000000'},{input:'-1.5',expected:'bff0000000000000'},{input:'0.5',expected:'3ff0000000000000'},{input:'NaN',expected:'nan'},{input:'Infinity',expected:'7ff0000000000000'}],
 sign:[{input:'-0',expected:'8000000000000000'},{input:'0',expected:'0000000000000000'},{input:'-Infinity',expected:'bff0000000000000'},{input:'NaN',expected:'nan'}],
 fround:[{input:'-0',expected:'8000000000000000'},{input:'16777217',expected:'4170000000000000'},{input:'16777219',expected:'4170000040000000'},{input:'Infinity',expected:'7ff0000000000000'},{input:'NaN',expected:'nan'}],
 min:[{inputs:[],expected:'7ff0000000000000'},{inputs:['0','-0'],expected:'8000000000000000'},{inputs:['-0','0'],expected:'8000000000000000'},{inputs:['NaN','1'],expected:'nan'}],
 max:[{inputs:[],expected:'fff0000000000000'},{inputs:['0','-0'],expected:'0000000000000000'},{inputs:['-0','0'],expected:'0000000000000000'},{inputs:['1','NaN'],expected:'nan'}],
 clz32:[{input:'0',expectedInteger:32},{input:'-1',expectedInteger:0},{input:'1',expectedInteger:31},{input:'4294967296',expectedInteger:32},{input:'NaN',expectedInteger:32}],
 imul:[{inputs:['4294967295','5'],expectedInteger:-5},{inputs:['2147483648','1'],expectedInteger:-2147483648},{inputs:['NaN','3'],expectedInteger:0}]
};
