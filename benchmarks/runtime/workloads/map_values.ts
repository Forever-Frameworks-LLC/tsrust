function workload(n: number, seed: number, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const values=new Map<number,number>([[i+seed,1],[i+seed+1,2],[i+seed,3]]);
  const first=values.get(i+seed)??0;
  values.set(i+seed+1,4);
  const found=values.has(i+seed+1);
  const removed=values.delete(i+seed);
  const again=values.delete(i+seed);
  const labels=new Map<number,string>([[1,String(input)],[2,"tail"]]);
  const copied=new Map(labels);
  labels.clear();
  {
   const text=copied.get(1);
   output.push([String(first),String(values.size),String(values.get(i+seed+1)??0),String(found),String(removed),String(again),String(labels.size),String(copied.size),String(text??"missing")]);
  }
  values.clear();
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
