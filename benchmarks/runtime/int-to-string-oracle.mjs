// Independent exact positional digits: no native/JavaScript formatting methods.
import {observeSliceTree} from './slice-observation.mjs';
export const intToStringIds=['int_to_string_decimal','int_to_string_binary','int_to_string_hex','int_to_string_base36'];
export const intToStringPool=[0,1,-1,2,-2,35,-35,255,-255,65535,-65535,2147483647,-2147483648,9007199254740991,-9007199254740991,4294967296];
const bases=[10,2,16,36],digits='0123456789abcdefghijklmnopqrstuvwxyz';
export function exactIntegerDigits(value,radix){if(!Number.isSafeInteger(value)||!Number.isInteger(radix)||radix<2||radix>36)throw Error('Exact safe integer and radix2..36 required');let remaining=BigInt(value),sign='';if(remaining<0n){sign='-';remaining=-remaining;}if(remaining===0n)return '0';const out=[],base=BigInt(radix);while(remaining){out.push(digits[Number(remaining%base)]);remaining/=base;}return sign+out.reverse().join('');}
export function validateIntToStringCase(id,n,seed,input){if(!intToStringIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!==undefined)throw Error('Bounded numeric-only formatting case required');}
export function expectedIntToStringValues(id,n,seed){validateIntToStringCase(id,n,seed);const radix=bases[intToStringIds.indexOf(id)],out=[];let at=seed%intToStringPool.length;for(let k=0;k<n;k++){out.push(exactIntegerDigits(intToStringPool[at],radix));at=(at+13)%intToStringPool.length;}return out;}
export function expectedIntToStringObservation(id,n,seed,input){validateIntToStringCase(id,n,seed,input);return observeSliceTree(expectedIntToStringValues(id,n,seed));}
