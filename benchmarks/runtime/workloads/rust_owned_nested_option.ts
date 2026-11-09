type Maybe<T>={kind:"none"}|{kind:"some";value:T};function nested(value:number):Maybe<Maybe<string[]>>{switch(value%3){case 0:return{kind:"none"};case 1:return{kind:"some",value:{kind:"none"}};default:return{kind:"some",value:{kind:"some",value:["alpha","beta"]}};}}
function workload(n: number, seed: number): number {
  let checksum = 0;
  let i = 0;
  while (i < n) {
    const result = nested(seed + i);
    let observed = 1;
    if (result.kind === "some") {
      const inner = result.value; observed = 2;
      if (inner.kind === "some") { observed = 3; for (const word of inner.value) { observed = observed + word.length; } }
    }
    checksum = (checksum * 131 + observed) % 1000000007;
    i = i + 1;
  }
  return checksum;
}
