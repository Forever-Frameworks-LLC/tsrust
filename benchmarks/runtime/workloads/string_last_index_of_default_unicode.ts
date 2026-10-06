type int = number;
// Standard scalar UTF8 mapping, consistent with the accepted string-search counterpart.
function coordinates(value: string): [number[], number[]] {
    const byteToUnit: number[] = [0], unitToByte: number[] = [0];
    const encoder = new TextEncoder(); let bytes = 0, units = 0;
    for (const scalar of value) {
        const code = scalar.codePointAt(0)!;
        if (code >= 0xD800 && code <= 0xDFFF) throw new Error('Valid Unicode required');
        bytes += encoder.encode(scalar).length; units += scalar.length;
        byteToUnit[bytes] = units; unitToByte[units] = bytes;
    }
    return [byteToUnit, unitToByte];
}
function byteIndex(input: string, needle: string, unitToByte: number[]): number {
    const found = input.indexOf(needle);
    return found < 0 ? -1 : unitToByte[found];
}
function searchResult(input: string, needle: string, position: number | undefined, byteToUnit: number[], unitToByte: number[]): number {
    if (position === undefined) {
        const found = input.lastIndexOf(needle);
        return found < 0 ? -1 : unitToByte[found];
    }
    const bytePosition = Math.max(0, Math.min(byteToUnit.length - 1, position));
    const unitPosition = byteToUnit[bytePosition];
    if (unitPosition === undefined) throw new Error('UTF8 boundary required');
    const found = input.lastIndexOf(needle, unitPosition);
    return found < 0 ? -1 : unitToByte[found];
}
function workload(n: int, seed: int, input: string): int[] {
    const [byteToUnit, unitToByte] = coordinates(input);
    const needles: string[] = ["", "a", "aa", "aba", "é", "😀", "é", "missing\u0000", "a\u0000b"];
    const output: int[] = []; let i = 0;
    while (i < n) {
        const needle = needles[(i * 17 + seed) % 9];
        const at = searchResult(input, needle, undefined, byteToUnit, unitToByte);
        output.push(at + 1); i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "aaababaé😀ababaé😀é😀Z"); }
