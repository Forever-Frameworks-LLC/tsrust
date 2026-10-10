export {};
type int = number;
type ByteError = { code: string; offset?: int; };
type ByteResult = { kind: "Ok"; value: Uint8Array; } | { kind: "Err"; error: ByteError; };
function fromArray(values: readonly int[]): ByteResult {
    for (let offset = 0; offset < values.length; offset += 1) {
        if (!Number.isInteger(values[offset]) || values[offset] < 0 || values[offset] > 255) {
            return { kind: "Err", error: { code: "byte-range", offset } };
        }
    }
    return { kind: "Ok", value: Uint8Array.from(values) };
}
function length(bytes: Uint8Array): int { return bytes.length; }
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    const construct = fromArray;
    const observe = length;
    let i = 0;
    while (i < n) {
        const byte = (i + seed) % 256;
        const values = [0, byte, 255];
        let first = -1;
        let second = -1;
        const result = construct(values);
        switch (result.kind) {
            case "Ok": first = observe(result.value); second = observe(result.value); break;
            case "Err": first = -2; second = -2; break;
        }
        let empty = -1;
        const emptyResult = fromArray([]);
        switch (emptyResult.kind) {
            case "Ok": empty = length(emptyResult.value); break;
            case "Err": empty = -2; break;
        }
        let offset = -1;
        let codeLength = -1;
        const invalid = i % 2 === 0 ? -1 : 256;
        const invalidResult = fromArray([byte, invalid, 0]);
        switch (invalidResult.kind) {
            case "Ok": offset = -2; codeLength = length(invalidResult.value); break;
            case "Err": offset = invalidResult.error.offset ?? -3; codeLength = invalidResult.error.code.length; break;
        }
        output.push([first, second, values[1], empty, offset, codeLength]);
        i += 1;
    }
    return output;
}
