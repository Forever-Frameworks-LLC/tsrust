type int = number;
function clone<T>(value:T):T{return value;}
function workload(n: int, seed: int, input: string): string[][] {
 const output: string[][] = [];  let i = 0;
 while (i < n) { const mode = (i + seed) % 3; const base = (i * 13 + seed) % 97; const count = mode === 0 ? 0 : mode === 1 ? 1 : 5;
 const nils:null[] = []; const absents:undefined[] = []; let at = 0; while(at < count){nils.push(null);absents.push(undefined);at = at + 1;} const nilResult = [...nils]; const absentResult = [...absents]; const nilText:string[] = []; const absentText:string[] = []; at = 0; while(at < nilResult.length){const kind = typeof nilResult[at];nilText.push(kind.concat(":",String(nilResult[at])));at = at + 1;} at = 0; while(at < absentResult.length){const kind = typeof absentResult[at];absentText.push(kind.concat(":",String(absentResult[at])));at = at + 1;} output.push(nilText);output.push(absentText); i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
