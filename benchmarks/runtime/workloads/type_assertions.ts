type Row={value:number;};
function workload(n: number, seed: number, input: string): string[][]{
 const output:string[][]=[];let i=0;
 while(i<n){
  const value=(i*13+seed)%97;
  const values=[] as number[];values.push(value);
  const checked=value satisfies number;
  const widened=value as number;
  const alternative=value as number|string;
  const row={value:checked} satisfies Row;
  let rendered="";
  if(typeof alternative==="number"){rendered=String(alternative);}
  output.push([String(values[0]),String(widened+0.5),String(row.value),input+rendered]);
  i=i+1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
