import {observeSliceTree} from './slice-observation.mjs';
export function expectedNativeByteAccessValues(id, n, seed, input) {
    if (id !== 'native_byte_access' || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'bytes') throw Error('Native byte access case');
    return Array.from({length:n}, (_,i) => { const byte=(i+seed)%256, changed=(byte+1)%256; return [byte,changed,1,changed,changed,11,0,10,1,11,3,3]; });
}
export function expectedNativeByteAccessObservation(id,n,seed,input) { return observeSliceTree(expectedNativeByteAccessValues(id,n,seed,input)); }
