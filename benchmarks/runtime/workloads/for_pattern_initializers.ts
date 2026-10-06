type int=number;
type Bounds={start:int;end:int;};
function bounds(base:int):Bounds{return {start:base%3,end:base%3+3};}
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;const words=[input.repeat(1),String(base)];
  let sum=0;for(let {start:k,end}=bounds(base);k<end;k++){if(k===base%3+1){continue;}sum+=k;}
  let count=0;for(let [j,stop=2]=[0];j<stop;j++){count+=j+1;}
  let text="";for(const [first,...tail]=words;tail.length<3;tail.push("x")){text+=first+String(tail.length);}
  output.push([String(sum),String(count),text,String(words.length)]);
  i=i+1;
 }return output;
}
function main(){const result=workload(3,0,"abc");}
