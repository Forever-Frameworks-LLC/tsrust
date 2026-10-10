export {};
type int = number;
type ByteError = { code: string; offset?: int; };
type Result<T> = { kind: "Ok"; value: T; } | { kind: "Err"; error: ByteError; };
function fromArray(values: readonly int[]): Result<Uint8Array> {
    for (let offset = 0; offset < values.length; offset += 1) {
        if (!Number.isInteger(values[offset]) || values[offset] < 0 || values[offset] > 255) return { kind: "Err", error: { code: "byte-range", offset } };
    }
    return { kind: "Ok", value: Uint8Array.from(values) };
}
function at(bytes: Uint8Array, index: int): Result<int> {
    if (index < 0 || index >= bytes.length) return { kind: "Err", error: { code: "index-range", offset: index } };
    return { kind: "Ok", value: bytes[index] };
}
function set(bytes: Uint8Array, index: int, value: int): Result<void> {
    if (index < 0 || index >= bytes.length) return { kind: "Err", error: { code: "index-range", offset: index } };
    if (value < 0 || value > 255) return { kind: "Err", error: { code: "byte-range", offset: index } };
    bytes[index] = value;
    return { kind: "Ok", value: undefined };
}
function length(bytes: Uint8Array): int { return bytes.length; }
function read(bytes: Uint8Array): int {
    const result = at(bytes, 1);
    switch (result.kind) { case "Ok": return result.value; case "Err": return -1; }
}
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    const write = set;
    let i = 0;
    while (i < n) {
        const byte = (i + seed) % 256;
        const changed = (byte + 1) % 256;
        const result = fromArray([0, byte, 255]);
        switch (result.kind) {
            case "Ok": {
                const bytes = result.value;
                const initial = read(bytes);
                let success = -1;
                const written = write(bytes, 1, changed);
                switch (written.kind) { case "Ok": success = 1; break; case "Err": success = -2; break; }
                const updated = read(bytes);
                let indexCode = -1;
                let indexOffset = -9;
                const invalidIndex = set(bytes, -1, 256);
                switch (invalidIndex.kind) { case "Ok": indexCode = -2; break; case "Err": indexCode = invalidIndex.error.code.length; indexOffset = invalidIndex.error.offset ?? -3; break; }
                const afterIndex = read(bytes);
                let byteCode = -1;
                let byteOffset = -9;
                const invalidByte = set(bytes, 1, 256);
                switch (invalidByte.kind) { case "Ok": byteCode = -2; break; case "Err": byteCode = invalidByte.error.code.length; byteOffset = invalidByte.error.offset ?? -3; break; }
                const afterByte = read(bytes);
                let readCode = -1;
                let readOffset = -9;
                const invalidRead = at(bytes, 3);
                switch (invalidRead.kind) { case "Ok": readCode = -2; break; case "Err": readCode = invalidRead.error.code.length; readOffset = invalidRead.error.offset ?? -3; break; }
                output.push([initial, updated, success, afterIndex, afterByte, indexCode, indexOffset + 1, byteCode, byteOffset, readCode, readOffset, length(bytes)]);
                break;
            }
            case "Err": output.push([-1000000]); break;
        }
        i += 1;
    }
    return output;
}
