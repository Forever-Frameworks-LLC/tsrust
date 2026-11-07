type Row={label:string;value:number};function rows(value:Row[]):Row[]{return value;}
function workload(n: number, seed: number): number {
  let checksum = 0;
  let i = 0;
  while (i < n) {
    const a: Row = {label: "alpha", value: seed + i};
    const b: Row = {label: "beta", value: seed + i + 1};
    const returned = rows([a, b]);
    let observed = 0;
    for (const row of returned) { observed = observed + row.label.length + row.value; }
    checksum = (checksum * 131 + observed) % 1000000007;
    i = i + 1;
  }
  return checksum;
}
