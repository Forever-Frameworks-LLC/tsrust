import {rationalBits,tokensObservation} from './number-oracle.mjs';
function floorSqrt(value) {
 if(value<2n)return value;
 let estimate=1n<<BigInt((value.toString(2).length+1)>>1);
 for(;;){const next=(estimate+value/estimate)/2n;if(next>=estimate)return estimate;estimate=next;}
}
// Independent nearest-even IEEE sqrt, using exact integer midpoint comparisons.
export function sqrtRationalBits(n,d=1n,negativeZero=false) {
 if(n<0n)return 'nan';if(n===0n)return rationalBits(0n,1n,negativeZero);
 let exponent=n.toString(2).length-d.toString(2).length;
 if(exponent>=0?n<(d<<BigInt(exponent)):(n<<BigInt(-exponent))<d)exponent--;
 exponent=exponent>>1;
 const shift=exponent< -1022?1074:52-exponent;
 const numerator=shift>=0?n<<BigInt(2*shift):n;
 const denominator=shift>=0?d:d<<BigInt(-2*shift);
 let mantissa=floorSqrt(numerator/denominator);
 const midpoint=denominator*(2n*mantissa+1n)**2n;
 if(4n*numerator>midpoint||4n*numerator===midpoint&&mantissa%2n===1n)mantissa++;
 if(exponent< -1022)return mantissa.toString(16).padStart(16,'0');
 if(mantissa===1n<<53n){mantissa>>=1n;exponent++;}
 const bits=exponent>1023?0x7ff0000000000000n:(BigInt(exponent+1023)<<52n)|(mantissa-(1n<<52n));
 return bits.toString(16).padStart(16,'0');
}
export function expectedMathBits(method,value) {
 if(!['abs','floor','ceil','trunc','sqrt'].includes(method))throw new Error('Unknown Math method');
 if(value.kind==='nan')return 'nan';
 if(value.kind==='infinity')return method==='sqrt'&&value.negative?'nan':method==='abs'?'7ff0000000000000':value.negative?'fff0000000000000':'7ff0000000000000';
 const n=value.numerator,d=value.denominator??1n;
 if(method==='sqrt')return sqrtRationalBits(n,d,value.negativeZero);
 if(method==='abs')return rationalBits(n<0n?-n:n,d);
 let result=n/d;
 if(method==='floor'&&n<0n&&n%d!==0n)result--;
 if(method==='ceil'&&n>0n&&n%d!==0n)result++;
 return rationalBits(result,1n,result===0n&&(n<0n||value.negativeZero));
}
// Decode finite IEEE tokens into exact dyadic rationals, then round each
// supporting accumulation step independently; do not replace ordered f64 sums.
export function finiteBitsRational(token) {
 const bits=BigInt('0x'+token),sign=(bits>>63n)?-1n:1n;
 const exponent=Number((bits>>52n)&2047n),fraction=bits&((1n<<52n)-1n);
 if(exponent===2047)throw new Error('Finite IEEE token required');
 const significand=exponent===0?fraction:(1n<<52n)|fraction;
 const power=exponent===0?-1074:exponent-1023-52;
 return power>=0?{numerator:sign*(significand<<BigInt(power)),denominator:1n}:{numerator:sign*significand,denominator:1n<<BigInt(-power)};
}
export function addFiniteBits(left,right) {
 const a=finiteBitsRational(left),b=finiteBitsRational(right),denominator=a.denominator>b.denominator?a.denominator:b.denominator;
 return rationalBits(a.numerator*(denominator/a.denominator)+b.numerator*(denominator/b.denominator),denominator);
}
export function expectedMath(id,n,seed) {
 if(![n,seed].every(value=>Number.isSafeInteger(value)&&value>=0&&value<=1000000))throw new Error('Bounded Math inputs required');
 const method=id.replace(/^math_/,'');if(!['abs','floor','ceil','trunc','sqrt'].includes(method))throw new Error('Unknown Math workload');
 if(method==='sqrt') {
  const inputRationals=[[0n,1n],[1n,4n],[1n,1n],[2n,1n],[3n,1n],[25n,4n],[10n,1n],[25n,1n]];
  const results=inputRationals.map(([numerator,denominator])=>sqrtRationalBits(numerator,denominator));
  let sum=rationalBits(BigInt(seed));
  for(let i=0;i<n;i++)sum=addFiniteBits(sum,results[(i*13+seed)%8]);
  return tokensObservation([sum]);
 }
 // Exact quarter-unit expected outputs for the independently specified table.
 const outputs=method==='sqrt'?[0n,2n,4n,6n,8n,10n,12n,20n]:method==='abs'?[10n,7n,1n,0n,0n,1n,7n,10n]:method==='floor'?[-12n,-8n,-4n,0n,0n,0n,4n,8n]:method==='ceil'?[-8n,-4n,0n,0n,0n,4n,8n,12n]:[-8n,-4n,0n,0n,0n,0n,4n,8n];
 let sum=BigInt(seed)*4n;
 for(let i=0;i<n;i++)sum+=outputs[(i*13+seed)%8];
 return tokensObservation([rationalBits(sum,4n)]);
}
