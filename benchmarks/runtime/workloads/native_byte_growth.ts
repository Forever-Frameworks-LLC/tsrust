export {};
type int = number;
type Bytes = { data: Uint8Array };
type Result<T> = { kind: "Ok"; value: T } | { kind: "Err"; error: { code: string } };
function alloc(length: int, fill: int = 0): Result<Bytes> {
    if (!Number.isSafeInteger(length) || length < 0) return {kind:"Err",error:{code:"length-range"}};
    if (!Number.isInteger(fill) || fill < 0 || fill > 255) return {kind:"Err",error:{code:"byte-range"}};
    return {kind:"Ok",value:{data:new Uint8Array(length).fill(fill)}};
}
function resize(bytes: Bytes, length: int, fill: int = 0): Result<void> {
    if (!Number.isSafeInteger(length) || length < 0) return {kind:"Err",error:{code:"length-range"}};
    if (!Number.isInteger(fill) || fill < 0 || fill > 255) return {kind:"Err",error:{code:"byte-range"}};
    const next = new Uint8Array(length);
    next.set(bytes.data.subarray(0, Math.min(length, bytes.data.length)));
    if (length > bytes.data.length) next.fill(fill, bytes.data.length);
    bytes.data = next;
    return {kind:"Ok",value:undefined};
}
function length(bytes: Bytes): int { return bytes.data.length; }
function read(bytes: Bytes, index: int): int { return index >= 0 && index < bytes.data.length ? bytes.data[index] : -1; }
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    const change = resize;
    let i = 0;
    while (i < n) {
        const byte = (i + seed) % 256;
        const result = alloc(2, byte);
        switch(result.kind) {
            case "Ok": {
                const bytes = result.value;
                const initial = read(bytes, 0);
                const grown = change(bytes, 4);
                const growth = grown.kind === "Ok" ? 1 : -2;
                const zero = read(bytes, 3);
                const failed = resize(bytes, 0, 256);
                const invalid = failed.kind === "Err" ? failed.error.code.length : -2;
                const preserved = length(bytes);
                const shrunk = resize(bytes, 1);
                const shrink = shrunk.kind === "Ok" ? 1 : -2;
                const regrown = resize(bytes, 3, 255);
                const regrow = regrown.kind === "Ok" ? 1 : -2;
                output.push([initial, growth, zero, invalid, preserved, shrink, regrow, read(bytes, 0), read(bytes, 1), read(bytes, 2), length(bytes)]);
                break;
            }
            case "Err": output.push([-1000000]); break;
        }
        i += 1;
    }
    return output;
}
