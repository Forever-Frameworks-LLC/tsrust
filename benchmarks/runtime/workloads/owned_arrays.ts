type Entry = {name: string; score: number};
function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const entries: Entry[] = [{name: " Pat ", score: i}, {name: "Sam", score: seed}];
  entries.push({name: " Jo ", score: i + seed}); entries[1].score = i + 1;
  for (const entry of entries) { sum += entry.score; sum++; }
  const saved = {name: entries[1].name, score: entries[1].score}; sum += saved.score;
  const names = [" A "]; names.push(" B "); sum += names.length;
 } return sum;
}
