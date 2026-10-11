function byteWindow(owner: Uint8Array): Uint8Array { return owner.subarray(0, 2); }
function first(part: Uint8Array): number { return part[0]; }
function workload(n: number, seed: number, input: string): number[][] {
    const output: number[][] = []; let i = 0; while (i < n) {
        const byte = (i + seed) % 256; const owner = Uint8Array.from([byte, 255]);
        const values: Uint8Array[] = []; let count = 0; while (count < 8) { values.push(byteWindow(owner)); count++; }
        values.unshift(byteWindow(owner)); let read = 0; { const selected = values[4]; if (selected !== undefined) { read = first(selected); } }
        let traversed = 0; for (const part of values) { traversed += first(part); }
        const front = values.shift()!; const back = values.pop()!; let sum = first(front) + first(back);
        while (values.length > 0) { const part = values.shift()!; sum += first(part); }
        values.push(byteWindow(owner)); values.length = 0; output.push([count + 1, read, sum, values.length, traversed]);
        i++;
    } return output;
}
function main() { }
