import {observeSliceTree} from './slice-observation.mjs';

export function expectedResultMatchValues(id, n, seed, input) {
    if (id !== 'result_match' || !Number.isSafeInteger(n) || n < 0 || n > 2048 || !Number.isSafeInteger(seed) || seed < 0 || seed > 1000000 || input !== 'results') throw Error('Result match case');
    return Array.from({length: n}, (_, i) => {
        const value = i % 17 + seed % 19 + 1;
        const inspected = value % 3 === 0 ? (value + 5) * 3 : value * 2;
        const owned = value + (value % 3 === 0 ? 16 : 7);
        const called = value + (value % 4 === 0 ? 0 : 1);
        return [inspected, inspected, owned, called, called + 1];
    });
}

export function expectedResultMatchObservation(id, n, seed, input) {
    return observeSliceTree(expectedResultMatchValues(id, n, seed, input));
}
