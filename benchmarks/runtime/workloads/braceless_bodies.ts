type int=number;
function grade(value:int):string{
 if(value<20)return "low";
 else if(value<60)return "mid";
 else return "high";
}
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;let sum=0,odd=0,text="";
  for(let k=0;k<base%5+1;k++)sum+=k;
  for(const part of [input.repeat(1),String(base)])text+=part;
  for(let a=0;a<3;a++)for(let b=0;b<3;b++)if(a===b)odd+=1;else if(a+b===2)odd+=10;
  let down=base%4+1;
  while(down>1)down--;
  do down+=2;while(down<4);
  output.push([grade(base),String(sum),text,String(odd),String(down)]);
  i=i+1;
 }return output;
}
function main(){const result=workload(3,0,"abc");}
