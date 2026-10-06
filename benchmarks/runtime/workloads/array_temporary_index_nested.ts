type int = number;
function workload(n: int, seed: int): int[][] {
    const output: int[][] = []; let i = 0;
    while (i < n) {
        output.push(make_nested(seed, i)[index_for(i, seed)]);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); console.log(result.length); }
function make_nested(seed: int, i: int): int[][] { return [[seed + i, seed + i + 1], [], [i + 2, seed + 3, 4], [seed + 5]]; }
function index_for(i: int, seed: int): int { return (i * 3 + seed) % 4; }
