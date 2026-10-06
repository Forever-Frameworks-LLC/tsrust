// Independent index-domain normalization and arithmetic tree fingerprints.
const starts=[0,0,1,-1,-8,-80,0,1,3,-3,100,-100,3,-2,-2,0],ends=[0,0,0,0,0,0,0,3,3,-1,200,-80,1,-2,200,-1],modes=[0,1,1,1,1,1,2,2,2,2,2,2,2,2,2,2];
export function normalizeBound(bound,length){const b=BigInt(bound),size=BigInt(length);return Number(b<0n?(size+b<0n?0n:size+b):b>size?size:b);}
export function expectedSliceQueries(length){return modes.map((mode,index)=>{const first=mode===0?0:normalizeBound(starts[index],length),last=mode<2?length:normalizeBound(ends[index],length);return {mode,first,last:last<first?first:last};});}
export function expectedSlices(id,n,seed,input='élan 😀 e\u0301') {
 if(![n,seed].every(v=>Number.isSafeInteger(v)&&v>=0&&v<=1000000)||n>2000)throw new Error('Bounded slice inputs required');
 for(const scalar of input){const code=scalar.codePointAt(0);if(code>=0xd800&&code<=0xdfff)throw new Error('Valid Unicode input required');}
 const values=id==='array_slice_int'?[seed,seed+1,2,3,seed+4,5,6,7]:id==='array_slice_number'?[seed+.25,-0,2.25,seed+1.5,0,4.5,5.25,-2.5]:id==='array_slice_string'?[input,'Pat',input,'','é','😀','e\u0301','a\0b']:id==='array_slice_nested'?[[seed,seed+1],[],[2,3,4],[seed+5],[7,8],[9],[],[11,12,13]]:undefined;
 if(!values)throw new Error('Unknown array slice draft');
 const queries=expectedSliceQueries(8),out=[];
 for(let i=0;i<n;i++){const query=queries[(i*13+seed)%16],row=[];for(let position=query.first;position<query.last;position++){const value=values[position];if(Array.isArray(value)){const copied=[];for(const item of value)copied.push(item);row.push(copied);}else row.push(value);}out.push(row);}
 return out;
}
// Bit tokens are built from exact bounded quarter rationals, not DataView.
function rationalToken(value){const negative=value<0||Object.is(value,-0);let numerator=BigInt(value<0?-value*4:value*4);if(numerator===0n)return negative?'8000000000000000':'0000000000000000';let exponent=numerator.toString(2).length-3;const shift=50-exponent;const mantissa=shift>=0?numerator<<BigInt(shift):numerator>>BigInt(-shift);const bits=(negative?1n<<63n:0n)|(BigInt(exponent+1023)<<52n)|(mantissa-(1n<<52n));return bits.toString(16).padStart(16,'0');}
export function expectedSliceObservation(id,n,seed,input) {
 const numbers=id==='array_slice_number';let hash=0n;const add=value=>{hash=(hash*131n+BigInt(value))%1000000007n;};
 function encode(value){if(Array.isArray(value)){add(256);const items=[];for(const item of value)items.push(encode(item));add(257);return '['+items.join(',')+']';}if(numbers){const token=rationalToken(value),bits=BigInt('0x'+token);for(let shift=56n;shift>=0n;shift-=8n)add((bits>>shift)&255n);return token;}if(typeof value==='number'){add(value);return String(value);}add(258);const codes=[];for(const scalar of value){const code=scalar.codePointAt(0);add(code);codes.push(code);}add(259);return 's('+codes.join(',')+')';}
 const shape=encode(expectedSlices(id,n,seed,input));return {checksum:Number(hash),exactBits:(numbers?'m:':'t:')+shape};
}
