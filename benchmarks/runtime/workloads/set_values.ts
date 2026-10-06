function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const values=new Set<number>([i+seed,i+seed+1,i+seed]);
  const found=values.has(i+seed);
  values.delete(i+seed);
  values.add(i+seed);
  const keys=Array.from(values.values());
  const labels=new Set<string>([String(input),"tail",String(input)]);
  let order="";
  for(const key of values){order=order+String(key)+":";}
  const copied=new Set(labels);
  const labelKeys=Array.from(labels.keys());
  labels.clear();
  output.push([String(values.size),String(found),keys.join(","),String(copied.size),String(labels.size),order,labelKeys.join("|")]);
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
