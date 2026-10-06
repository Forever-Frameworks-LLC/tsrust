function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const values=new Map<number,string>([[i+seed,String(input)],[i+seed+1,"tail"]]);
  values.delete(i+seed);
  values.set(i+seed,String(input));
  const keys=Array.from(values.keys());
  const labels=Array.from(values.values());
  const pairs=Array.from(values.entries());
  values.clear();
  output.push([keys.join(","),labels.join("|"),String(pairs[0][0]),String(pairs[0][1]),String(pairs[1][0]),String(pairs[1][1]),String(values.size)]);
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
