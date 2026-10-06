type int = number;
type Trace = { predicates: int; left: int; right: int; };
function workload(n: int, seed: int): number[][] {
    const output: number[][] = []; const trace: Trace = { predicates: 0, left: 0, right: 0 }; let i = 0;
    while (i < n) {
        const size: int = (i * 7 + seed * 2) % 4; const at: int = (i * 13 + seed * 2) % 8;
        const selected: number[] = predicate(trace, (i + seed) % 3 === 0) ? leftValue(trace, at, size) : rightValue(trace, at, size);
        output.push(selected);
        i = i + 1;
    }
    output.push([Number(trace.predicates), Number(trace.left), Number(trace.right)]);
    return output;
}
function predicate(trace: Trace, flag: boolean): boolean { trace.predicates = trace.predicates + 1; return flag; }
function leftValue(trace: Trace, at: int, size: int): number[] { trace.left = trace.left + 1; const values: number[] = []; const pool: number[] = [0.0, -0.0, 0.5, -0.5, 1.0 / 0.0, -1.0 / 0.0, 0.0 / 0.0, 4.0]; let j = 0; while (j < size) { values.push(pool[(at + j + 0) % 8]); j = j + 1; } return values; }
function rightValue(trace: Trace, at: int, size: int): number[] { trace.right = trace.right + 1; const values: number[] = []; const pool: number[] = [0.0, -0.0, 0.5, -0.5, 1.0 / 0.0, -1.0 / 0.0, 0.0 / 0.0, 4.0]; let j = 0; while (j < size) { values.push(pool[(at + j + 3) % 8]); j = j + 1; } return values; }
