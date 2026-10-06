type int = number;
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
function searchResult(input: string, needle: string, position: number | undefined, byteToUnit: number[], unitToByte: number[]): boolean {
    const bytePosition = position === undefined ? undefined : Math.max(0, Math.min(byteToUnit.length - 1, position));
    const unitPosition = bytePosition === undefined ? undefined : byteToUnit[bytePosition];
    if (bytePosition !== undefined && unitPosition === undefined) throw new Error('UTF-8 boundary required');
    const found = input.includes(needle, unitPosition);
    return found;
}
function workload(n: int, seed: int, input: string): int {
    const [byteToUnit, unitToByte] = coordinates(input);
    const needles: string[] = ["", "A", "é", "😀", "é", "absent\u0000", "é😀"];
    const positions: int[] = [-7, 0, 1, 3, 7, 8, 9, 11, 15, 16, 18, 22, 23, 999];
    let sum = seed; let i = 0;
    while (i < n) {
        const position = positions[(i * 13 + seed) % 14];
        let code = 0;
        if ((i + seed) % 2 === 0) {
            if (searchResult(input, needles[(i * 17 + seed) % 7], undefined, byteToUnit, unitToByte)) { code = 1; }
        } else {
            if (searchResult(input, needles[(i * 17 + seed) % 7], position, byteToUnit, unitToByte)) { code = 1; }
        }
        sum = (sum * 131 + code) % 1000000007;
        i = i + 1;
    } return sum;
}
function main() { console.log(workload(17, 7, "Aé😀Aé😀Aé😀Z")); }
