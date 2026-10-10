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
function view(bytes: Uint8Array, start: int, end: int): Result<Uint8Array> {
    if (start < 0 || start > bytes.length) return { kind: "Err", error: { code: "range", offset: start } };
    if (end < start || end > bytes.length) return { kind: "Err", error: { code: "range", offset: end } };
    return { kind: "Ok", value: bytes.subarray(start, end) };
}
const viewMut = view;
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
function copy(bytes: Uint8Array, start: int, end: int): Result<Uint8Array> {
    if (start < 0 || start > bytes.length) return { kind: "Err", error: { code: "range", offset: start } };
    if (end < start || end > bytes.length) return { kind: "Err", error: { code: "range", offset: end } };
    return { kind: "Ok", value: bytes.slice(start, end) };
}
function length(bytes: Uint8Array): int { return bytes.length; }
function first(part: Uint8Array): int {
    const result = at(part, 0);
    switch (result.kind) { case "Ok": return result.value; case "Err": return -1; }
}
function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    let i = 0;
    while (i < n) {
        const byte = (i + seed) % 256;
        const next = (byte + 1) % 256;
        const written = (byte + 2) % 256;
        const result = fromArray([0, byte, next, 255, 7]);
        switch (result.kind) {
            case "Ok": {
                const bytes = result.value;
                let size = -1;
                let head = -1;
                let tail = -1;
                let inner = -1;
                let copied = -1;
                const readable = view(bytes, 1, 4);
                switch (readable.kind) {
                    case "Ok": {
                        const part = readable.value;
                        size = length(part);
                        head = first(part);
                        const last = at(part, 2);
                        switch (last.kind) { case "Ok": tail = last.value; break; case "Err": tail = -2; break; }
                        const narrowed = view(part, 1, 2);
                        switch (narrowed.kind) { case "Ok": inner = first(narrowed.value); break; case "Err": inner = -2; break; }
                        const duplicate = copy(part, 0, 2);
                        switch (duplicate.kind) { case "Ok": copied = length(duplicate.value); break; case "Err": copied = -2; break; }
                        break;
                    }
                    case "Err": size = -2; break;
                }
                let success = -1;
                let writeCode = -1;
                let writeOffset = -9;
                const writable = viewMut(bytes, 1, 3);
                switch (writable.kind) {
                    case "Ok": {
                        const window = writable.value;
                        const stored = set(window, 1, written);
                        switch (stored.kind) { case "Ok": success = 1; break; case "Err": success = -2; break; }
                        const outside = set(window, 2, written);
                        switch (outside.kind) { case "Ok": writeCode = -2; break; case "Err": writeCode = outside.error.code.length; writeOffset = outside.error.offset ?? -3; break; }
                        break;
                    }
                    case "Err": success = -3; break;
                }
                let through = -1;
                const observed = at(bytes, 2);
                switch (observed.kind) { case "Ok": through = observed.value; break; case "Err": through = -2; break; }
                let rangeCode = -1;
                let rangeOffset = -9;
                const beyond = view(bytes, 4, 9);
                switch (beyond.kind) { case "Ok": rangeCode = -2; break; case "Err": rangeCode = beyond.error.code.length; rangeOffset = beyond.error.offset ?? -3; break; }
                let reversedOffset = -9;
                const reversed = view(bytes, 3, 1);
                switch (reversed.kind) { case "Ok": reversedOffset = -2; break; case "Err": reversedOffset = reversed.error.offset ?? -3; break; }
                output.push([size, head, tail, inner, copied, success, writeCode, writeOffset, through, rangeCode, rangeOffset, reversedOffset]);
                break;
            }
            case "Err": output.push([-1000000]); break;
        }
        i += 1;
    }
    return output;
}
