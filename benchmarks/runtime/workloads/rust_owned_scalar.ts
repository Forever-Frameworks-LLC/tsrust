function scalar(value:number):number{return value*3+1;}
function workload(n: number, seed: number): number {
  let checksum = 0;
  let i = 0;
  while (i < n) {
    const observed = scalar(seed + i);
    checksum = (checksum * 131 + observed) % 1000000007;
    i = i + 1;
  }
  return checksum;
}
