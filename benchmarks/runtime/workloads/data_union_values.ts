type UnionRow={value:number|string|boolean|null;owner:string|number[];};
function copyRow(value:UnionRow):UnionRow{return {value:typeof value.value==="string"?value.value.slice():value.value,owner:typeof value.owner==="string"?value.owner.slice():value.owner.slice()};}
function describe(value:number|string|boolean|null):string {
 if(typeof value==="string"){return value.toUpperCase();}
 return String(value??"missing");
}
function copiedText(value:string|number[]=[1]):string{return String(value.slice());}
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;const mode=(i+seed)%4;
  const source:UnionRow={owner:mode%2===0?input.slice():[base,base+1],value:mode===0?base:mode===1?input.repeat(base%3+1):mode===2?false:null};
  const copied=copyRow(source);copied.owner=[base+5];
  output.push([String(source.value),typeof source.value,String(source.owner),String(copied.owner),describe(source.value),copiedText(source.owner)]);
  i=i+1;
 } return output;
}
function main(){const result=workload(3,0,"abc");}
