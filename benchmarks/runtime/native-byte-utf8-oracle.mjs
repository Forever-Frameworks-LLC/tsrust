import {observeSliceTree} from './slice-observation.mjs';
export function expectedNativeByteUtf8Values(id,n,seed,input){
    if(id!=='native_byte_utf8'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='bytes')throw Error('Native byte UTF-8 case');
    const lengths=[5,6,3,4];
    return Array.from({length:n},(_,i)=>{const length=lengths[(i+seed)%4];return [length,1,4,1,4,1,length];});
}
export function expectedNativeByteUtf8Observation(id,n,seed,input){return observeSliceTree(expectedNativeByteUtf8Values(id,n,seed,input));}
