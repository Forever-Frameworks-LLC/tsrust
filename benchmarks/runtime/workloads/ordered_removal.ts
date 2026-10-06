type Entry = {value: number};
function workload(n: number, seed: number): number {
 let sum = 0;
 for (let i = 0; i < n; i++) {
  const values: number[] = []; for (let j = 0; j < 32; j++) values.push(seed + i + j);
  sum += values.splice(3, 1)[0]; sum += values.splice(values.length - 1, 1)[0]; sum += values.splice(0, 1)[0];
  let position = 1; for (const value of values) { sum += value * position; position++; }
  const entries: Entry[] = [{value: i}, {value: seed}], entry = entries.splice(0, 1)[0]; sum += entry.value;
  const rows = [[i, seed], [seed]], row = rows.splice(0, 1)[0]; sum += row[0] + row[1];
 } return sum;
}
