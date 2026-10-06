type int = number;
function clone<T>(value:T):T{return value;}
function integers(base: int, count: int): int[] { const values: int[] = []; let at = 0; while (at < count) { values.push(base + at); at = at + 1; } return values; }
function collect<T>(...values: T[]): T[] { return values; }
function text(input:string,value:int):string{return input.repeat(value % 3 + 1).concat(":",String(value));}
function workload(n: int, seed: int, input: string): string[][] {
 const output: string[][] = [];  let i = 0;
 while (i < n) { const mode = (i + seed) % 3; const base = (i * 13 + seed) % 97; const count = mode === 0 ? 0 : mode === 1 ? 1 : 5;
 const values:string[] = []; let at = 0; while(at < count){values.push(text(input,base + at));at = at + 1;} output.push(collect(...values)); i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
