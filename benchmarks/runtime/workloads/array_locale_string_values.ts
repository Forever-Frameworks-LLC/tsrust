type int = number;
type Row = { n: int; };
function workload(n: int, seed: int, input: string): string[][] {
 const output: string[][] = []; let i = 0;
 while (i < n) { const base = (i * 13 + seed) % 97; const whole = base * 1001;
 const ints=[whole,0-whole];const numbers:number[]=[1000.5+Number(base),-0.0,1.2345];
 const words=[String(input),"é😀",""];const rows:Row[]=[{n:base},{n:whole}];
 const nested:number[][]=[[1000.5,-0.0],[],[2000.2345+Number(base)]];
 output.push([ints.toLocaleString(),numbers.toLocaleString(),words.toLocaleString(),[true,false].toLocaleString(),[null,null].toLocaleString(),[undefined,undefined].toLocaleString(),rows.toLocaleString(),nested.toLocaleString()]);
 i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
