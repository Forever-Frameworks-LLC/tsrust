/// <reference lib="esnext.collection" />
function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const left=new Set<number>([i+seed+3,i+seed+1,i+seed+2]);
  const right=new Set<number>([i+seed+2,i+seed+4,i+seed+1]);
  const small=new Set<number>([i+seed+2,i+seed+1]);
  const names=new Set<string>([String(input),"tail"]);
  const more=new Set<string>(["tail","last"]);
  const copied=names.union(more);names.clear();let text="";
  copied.forEach((value)=>{text=text+value+"|";});
  output.push([Array.from(left.union(right).values()).join(","),Array.from(left.intersection(right).values()).join(","),Array.from(left.difference(right).values()).join(","),Array.from(left.symmetricDifference(right).values()).join(","),Array.from(left.intersection(small).values()).join(","),text]);
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
