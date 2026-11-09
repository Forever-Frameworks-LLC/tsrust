type RegexRangeBenchmark = {start: number; end: number};
type Captures = {match: RegexRangeBenchmark; groups: (RegexRangeBenchmark | undefined)[]};
function captureSnapshot(value: RegExpExecArray): Captures {
  const slots = value.indices!;
  return {match: {start: slots[0][0], end: slots[0][1]}, groups: slots.slice(1).map(group => group === undefined ? undefined : {start: group[0], end: group[1]})};
}
function captureWeight(value: Captures): number {
  let weight = value.match.start + value.match.end + 1;
  for (let index = 0; index < value.groups.length; index++) {
    const group = value.groups[index];
    if (group !== undefined) { weight += group.start + group.end + 1; }
  }
  return weight;
}

function workload(n: number, seed: number, input: string): number {
  const expression = new RegExp('([a-z]+)=([0-9]+)', 'dgu');
  let checksum = seed;
  let i = 0;
  while (i < n) {
    expression.lastIndex = 0;
    const found = expression.exec(input);
    if (found !== null) { const value = captureSnapshot(found); checksum += captureWeight(value); }
    i++;
  }
  return checksum;
}
