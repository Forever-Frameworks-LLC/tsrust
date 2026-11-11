function words(value:string[]):string[]{return value;}
function workload(n: number, seed: number): number {
  let checksum = 0;
  let i = 0;
  while (i < n) {
    const returned = words(["alpha", "beta", "gamma"]);
    let observed = seed + i;
    for (const word of returned) { observed = observed + word.length; }
    checksum = (checksum * 131 + observed) % 1000000007;
    i = i + 1;
  }
  return checksum;
}
