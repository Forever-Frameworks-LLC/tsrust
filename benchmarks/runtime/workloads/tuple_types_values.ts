type int = number;
function makePair(index:int,value:string):[int,string] { return [index,value.slice()]; }
function copyPair(value:[int,string]):[int,string] { return [value[0],value[1].slice()]; }
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;const text=input.repeat(base%3+1);
  const pair=makePair(base,text);const copy=copyPair(pair);
  pair[0]=base+1;pair[1]=input.slice();
  const rows:[int,string][]=[pair,copy];
  const mapped=rows.map((item:[int,string],index:int):[int,string]=>{return [item[0]+index,item[1].slice()];});
  output.push([String(rows[0][0]),rows[0][1].slice(),String(rows[1][0]),rows[1][1].slice(),String(mapped[1][0]),mapped[1][1].slice(),String(mapped[1].length),String(Array.isArray(mapped[0]))]);
  i=i+1;
 } return output;
}
function main(){const result=workload(3,0,"abc");}
