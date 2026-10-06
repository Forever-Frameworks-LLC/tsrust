type int = number;
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const k=i+seed;const flag=k%2===0;const nullable:int|undefined=flag?k:undefined;
  let text=String(input);text+=k;text+=flag;
  output.push([input+k,k+input,"fraction:"+(Number(k%4)/2.0),"flag:"+flag,"absent:"+null+undefined,"union:"+nullable,text]);i+=1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
