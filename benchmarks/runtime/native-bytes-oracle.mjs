import {observeSliceTree} from './slice-observation.mjs';

export function expectedNativeBytesValues(id, n, seed, input) {
    if (id !== 'native_bytes' || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'bytes') throw Error('Native byte case');
    return Array.from({length: n}, (_, i) => [3, 3, (i + seed) % 256, 0, 1, 10]);
}
export function expectedNativeBytesObservation(id, n, seed, input) {
    return observeSliceTree(expectedNativeBytesValues(id, n, seed, input));
}
