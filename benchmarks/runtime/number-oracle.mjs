// Exact rational references and independent IEEE bit construction.
const modulus=1000000007n;
function roundedQuotient(n,d) {
 const quotient=n/d,remainder=n%d,twice=remainder*2n;
 return quotient+(twice>d||(twice===d&&quotient%2n===1n)?1n:0n);
}
export function rationalBits(n,d=1n,negativeZero=false) {
 const negative=(n<0n)!==(d<0n)||n===0n&&negativeZero;
 n=n<0n?-n:n;d=d<0n?-d:d;
 if(d===0n)return n===0n?'nan':negative?'fff0000000000000':'7ff0000000000000';
 const sign=negative?1n<<63n:0n;
 if(n===0n)return sign.toString(16).padStart(16,'0');
 let exponent=n.toString(2).length-d.toString(2).length;
 if(exponent>=0?n<(d<<BigInt(exponent)):(n<<BigInt(-exponent))<d)exponent--;
 if(exponent>1023)return(sign|(0x7ffn<<52n)).toString(16).padStart(16,'0');
 let fraction,encoded;
 if(exponent< -1022){fraction=roundedQuotient(n<<1074n,d);encoded=sign|fraction;}
 else {
  const shift=52-exponent;
  fraction=shift>=0?roundedQuotient(n<<BigInt(shift),d):roundedQuotient(n,d<<BigInt(-shift));
  if(fraction===1n<<53n){fraction>>=1n;exponent++;}
  encoded=exponent>1023?sign|(0x7ffn<<52n):sign|(BigInt(exponent+1023)<<52n)|(fraction-(1n<<52n));
 }
 return encoded.toString(16).padStart(16,'0');
}
export function tokensObservation(tokens,array=false) {
 let hash=0n;
 for(const token of tokens) {
  const bits=token==='nan'?0x7ff8000000000000n:BigInt('0x'+token);
  for(let shift=56n;shift>=0n;shift-=8n)hash=(hash*131n+((bits>>shift)&255n))%modulus;
 }
 return {checksum:Number(hash),exactBits:(array?'a:':'n:')+tokens.join(',')};
}
const comparisonValues=['nan','positiveInfinity','negativeInfinity',0n,0n,6n,-9n,9n];
function ordered(a,b) {
 if(a==='nan'||b==='nan')return undefined;
 if(a===b)return 0;
 if(a==='positiveInfinity'||b==='negativeInfinity')return 1;
 if(a==='negativeInfinity'||b==='positiveInfinity')return -1;
 const difference=a-b;return difference<0n?-1:difference>0n?1:0;
}
export function referenceComparison(method,a,b) {
 const order=ordered(a,b);
 return method==='equal'?order===0:method==='not_equal'?order!==0:order===undefined?false:
 method==='less'?order===-1:method==='less_equal'?order!==1:method==='greater'?order===1:order!==-1;
}
export function expectedNumber(id,n,seed) {
 if(![n,seed].every(v=>Number.isSafeInteger(v)&&v>=0&&v<=1000000))throw new Error('Bounded number draft inputs required');
 const supported=['number_value','number_array_operations','number_from_int','number_from_boolean','number_parse_decimal',...['add','subtract','multiply','divide','remainder','negate','equal','not_equal','less','less_equal','greater','greater_equal'].map(name=>'number_'+name)];
 if(!supported.includes(id))throw new Error('Unknown number draft');
 if(id==='number_value')return tokensObservation([rationalBits(2n*(BigInt(n)+BigInt(seed))+1n,2n)]);
 const comparison=/^number_(equal|not_equal|less|less_equal|greater|greater_equal)$/.exec(id);
 if(comparison) {
  let sum=BigInt(seed);
  for(let i=0;i<n;i++)sum=(sum*131n+(referenceComparison(comparison[1],comparisonValues[(i*13+seed)%8],comparisonValues[(i*17+seed)%8])?1n:0n))%modulus;
  return {checksum:Number(sum)};
 }
 if(id==='number_array_operations') {
  const tokens=[];
  for(let i=0;i<n;i++)tokens.push(rationalBits((BigInt(seed)+BigInt(i))*4n+1n,4n),rationalBits(0n,1n,true),rationalBits(9n,4n),rationalBits(BigInt(i)*2n+1n,2n),rationalBits(0n,1n,true),rationalBits(BigInt(i)*2n+3n,2n),rationalBits(0n),rationalBits(1n));
  return tokensObservation(tokens,true);
 }
 const arithmetic=/^number_(add|subtract|multiply|divide|remainder|negate)$/.exec(id);
 const left=[0n,1n,-2n,3n,10n,-12n,0n],right=[2n,-2n,8n,-8n,16n,-16n,4n];
 let sixteenths=arithmetic?BigInt(seed)*16n:0n;
 for(let i=0;i<n;i++) {
  if(arithmetic) {
   const a=left[(i*13+seed)%7],b=right[(i*17+seed)%7],op=arithmetic[1];
   sixteenths+=op==='add'?4n*(a+b):op==='subtract'?4n*(a-b):op==='multiply'?a*b:op==='divide'?16n*a/b:op==='remainder'?4n*(a%b):-4n*a;
  } else if(id==='number_from_int')sixteenths+=16n*(BigInt((i*37+seed)%2001)-1000n);
  else if(id==='number_from_boolean')sixteenths+=(i+seed)%2===0?16n:0n;
  else if(id==='number_parse_decimal')sixteenths+=[0n,0n,40n,-50n,1600n,2n,-8n][(i*13+seed)%7];
  else throw new Error('Unknown number draft');
 }
 return tokensObservation([rationalBits(sixteenths,16n)]);
}
