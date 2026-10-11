function workload(n:number,seed:number,input:string):number[][]{
 const output:number[][]=[];
 for(let i=0;i<n;i++){
  const values=new Set<number>();
  for(let j=0;j<9;j++){values.add((i+seed+j)%256);}
  values.add((i+seed)%256);
  let count=0;let weighted=0;let first=-1;let last=-1;
  for(const value of Array.from(values).sort((a,b)=>a-b)){if(count===0){first=value;}count++;weighted+=value*count;last=value;}
  const copied=new Set(values);
  const key=(i+seed+4)%256;
  const found=values.has(key)?1:0;
  const removed=values.delete(key)?1:0;
  output.push([count,first,last,weighted,found,removed,values.size,copied.size]);
 }
 return output;
}
function main(){}
