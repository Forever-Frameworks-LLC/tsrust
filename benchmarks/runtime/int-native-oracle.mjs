// Independent BigInt expectations; no measured Number operator supplies its own oracle.
import fs from 'node:fs';
export const nativeIntDomains=JSON.parse(fs.readFileSync(new URL('int-native-domains.json',import.meta.url)));
const MIN=-(1n<<63n),MAX=(1n<<63n)-1n;
function checked(value){if(value<MIN||value>MAX)throw Error('Checked i64 range exceeded');return value;}
export function nativeIntSpec(id){const match=/^int_native_(compound_)?(.+)$/.exec(id);if(!match||!Object.hasOwn(nativeIntDomains.operations,match[2])||match[1]&&['unary_plus','bit_not'].includes(match[2]))throw Error('Unknown native int row');return{name:match[2],form:match[1]?'compound':['unary_plus','bit_not'].includes(match[2])?'unary':'binary'};}
export function validateNativeIntCase(id,n,seed){const spec=nativeIntSpec(id);if(!Number.isSafeInteger(n)||n<0||n>nativeIntDomains.maxIterations||!Number.isSafeInteger(seed)||seed<0||seed>nativeIntDomains.maxSeed)throw Error('Bounded iteration/seed required');return spec;}
export function referenceNativeInt(name,left,right=0n){const a=checked(BigInt(left)),b=BigInt(right);if(name==='unary_plus')return a;if(name==='bit_not')return checked(-a-1n);checked(b);
 if(name==='bit_and')return a&b;if(name==='bit_or')return a|b;if(name==='bit_xor')return a^b;
 if(name.startsWith('shift_')){if(b<0n||b>63n)throw Error('Shift count must be0..63');if(name==='shift_left')return BigInt.asIntN(64,a<<b);if(name==='shift_right')return a>>b;if(name==='shift_right_unsigned')return BigInt.asIntN(64,BigInt.asUintN(64,a)>>b);}
 if(name==='power'){if(b<0n||b>4294967295n)throw Error('Exponent must fit u32');let exponent=b,factor=a,result=1n;while(exponent>0n){if(exponent%2n===1n)result=checked(result*factor);exponent=exponent/2n;if(exponent>0n)factor=checked(factor*factor);}return result;}
 throw Error('Unknown native int operation');
}
export function expectedNativeIntValues(id,n,seed){const spec=validateNativeIntCase(id,n,seed),domain=nativeIntDomains.operations[spec.name],out=[];let first=seed%8,second=seed%8;for(let i=0;i<n;i++){const exact=referenceNativeInt(spec.name,domain.left[first],domain.right===null?0:domain.right[second]);if(exact<0n||exact>BigInt(Number.MAX_SAFE_INTEGER))throw Error('Natural nonnegative exact timing output required');out.push(Number(exact));first=(first+5)%8;second=(second+7)%8;}return out;}
export function expectedNativeIntObservation(id,n,seed){const values=expectedNativeIntValues(id,n,seed);let checksum=0n;const add=code=>{checksum=(checksum*131n+BigInt(code))%1000000007n;};add(256);for(const value of values)add(value);add(257);return{checksum:Number(checksum),exactBits:'t:['+values.join(',')+']'};}
export const nativeIntHandCells={
 unary_plus:[[0],[-1],[1],[-2147483648],[2147483647],[-65536],[65535],[1024]],
 bit_not:[[0],[1],[-1],[-2],[-2147483648],[2147483647],[-65536],[65535]],
 bit_and:[[0,0],[1,0],[1,1],[-1,15],[-7,3],[-2147483648,-1],[2147483647,170],[65535,43690]],
 bit_or:[[0,0],[1,0],[1,2],[-1,15],[-7,3],[-2147483648,1],[2147483647,0],[65535,43690]],
 bit_xor:[[0,0],[1,0],[1,1],[-1,15],[-7,3],[-2147483648,-1],[2147483647,170],[65535,43690]],
 shift_left:[[0,0],[1,0],[1,1],[-1,1],[-7,2],[536870911,2],[0,31],[-1073741824,1]],
 shift_right:[[0,0],[1,0],[1,1],[-1,31],[-7,2],[-2147483648,31],[2147483647,31],[65535,15]],
 shift_right_unsigned:[[0,0],[1,0],[1,1],[1073741823,31],[65535,15],[2147483647,31],[255,4],[16,1]],
 power:[[0,0],[0,1],[1,0],[2,10],[-2,3],[-2,4],[7,7],[3,16]]
};
