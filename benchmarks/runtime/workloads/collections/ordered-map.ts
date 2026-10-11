function workload(n:number,seed:number,input:string):number[][]{
 const output:number[][]=[];
 for(let i=0;i<n;i++){
  const values=new Map<number,number>();
  for(let j=0;j<9;j++){const key=(i+seed+j)%256;values.set(key,key*3+1);}
  values.set((i+seed)%256,999);
  let count=0;let weighted=0;let sum=0;let first=-1;let last=-1;
  for(const entry of Array.from(values).sort((a,b)=>a[0]-b[0])){const key=entry[0];if(count===0){first=key;}count++;weighted+=key*count;sum+=entry[1];last=key;}
  const copied=new Map(values);const key=(i+seed+4)%256;
  const selected=values.get(key)!;const removed=values.delete(key)?1:0;
  output.push([count,first,last,weighted,sum,selected,removed,values.size,copied.size]);
 }
 return output;
}
function main(){}
