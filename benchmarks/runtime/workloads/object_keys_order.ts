type KeyRow={alpha:number;beta:string;gamma:boolean;};
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;
  const source:KeyRow=(i+seed)%2===0?{gamma:true,alpha:base,beta:input.slice()}:{beta:input.slice(),gamma:false,alpha:base};
  const names=Object.keys(source);const copied={...source};const copiedNames=Object.keys(copied);const textNames=Object.keys(input);
  const original=names.join("|");names[0]="changed";
  output.push([original,copiedNames.join("|"),names.join("|"),textNames.join("|"),String(source.alpha)]);
  i=i+1;
 } return output;
}
function main(){const result=workload(3,0,"abc");}
