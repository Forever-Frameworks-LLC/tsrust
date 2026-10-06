type int = number;
function workload(n: int, seed: int, input: string): boolean[][] {
 const output: boolean[][] = [];  let i = 0;
 while (i < n) { const sides = Number((i * 13 + seed) % 97 + 1); const first = Math.random(); const second = Math.random(); const roll = Math.floor(first * sides);
 output.push([first >= 0.0 && first < 1.0, second >= 0.0 && second < 1.0, first !== second, roll >= 0.0 && roll < sides]); i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
