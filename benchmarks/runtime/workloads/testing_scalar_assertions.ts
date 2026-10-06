/// <reference types="node" />
import {ok,strictEqual,notStrictEqual} from "node:assert/strict";
type int = number;
function workload(n: int, seed: int): int {
    const values: int[] = [0, 1, 4, 9, 16, 25, 36, 49];
    let checksum = 0;
    let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 8;
        const actual = values[at];
        const expected = at * at;
        ok(actual >= 0);
        strictEqual(actual, expected);
        notStrictEqual(actual, expected + 1);
        checksum = (checksum * 131 + actual) % 1000000007;
        i = i + 1;
    }
    return checksum;
}
function main() { console.log(workload(17, 3)); }
