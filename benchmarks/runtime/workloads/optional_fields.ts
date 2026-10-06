type int=number;
type Point = { x: int; y?: int; };
function width(point: Point): int { return point.x + (point.y ?? 0); }
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while(i<n){
  const k=i+seed;
  const flat:Point={x:k};
  const full:Point={x:k,y:k%7};
  let moving:Point={x:1};
  moving.y=k%3;
  const {y:taken=9}=flat;
  output.push([String(width(flat)),String(width(full)),String(width(moving)),String(taken),String(input)]);i+=1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
