import {observeSliceTree} from './slice-observation.mjs';
export function expectedNativeByteCloneValues(id,n,seed,input){
    if(id!=='native_byte_clone'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='bytes')throw Error('Native byte clone case');
    return Array.from({length:n},(_,i)=>{const byte=(i+seed)%256;return [byte,7,byte,byte,(byte+1)%256,3,3];});
}
export function expectedNativeByteCloneObservation(id,n,seed,input){return observeSliceTree(expectedNativeByteCloneValues(id,n,seed,input));}
