function splitRanges(expression: RegExp, text: string): {start: number; end: number}[] {
  let start = 0;
  const values: {start: number; end: number}[] = [];
  for (const match of text.matchAll(expression)) {
    values.push({start, end: match.index});
    start = match.index + match[0].length;
  }
  values.push({start, end: text.length});
  return values;
}

function workload(n: number, seed: number, input: string): number {
  const expression = new RegExp('(?:[a-z]+)=(?:[0-9]+)', 'dgu');
  let checksum = seed;
  let i = 0;
  while (i < n) {
    expression.lastIndex = 0;
    const values = splitRanges(expression, input);
    for (const value of values) { checksum += value.start + value.end + 1; }
    i++;
  }
  return checksum;
}
