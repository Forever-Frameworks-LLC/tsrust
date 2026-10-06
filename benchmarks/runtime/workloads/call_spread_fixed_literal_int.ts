type int = number;
function clone<T>(value:T):T{return value;}
function fixed(left:int,middle:int,right:int):int[]{return [left,middle,right];}
function workload(n: int, seed: int, input: string): int[][] {
 const output: int[][] = [];  let i = 0;
 while (i < n) { const mode = (i + seed) % 3; const base = (i * 13 + seed) % 97; const count = mode === 0 ? 0 : mode === 1 ? 1 : 5;
 output.push(fixed(...[base, base + mode + 1, base + 2])); i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
