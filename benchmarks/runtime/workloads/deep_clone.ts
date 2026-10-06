type Details = {name: string; scores: number[]}; type User = {details: Details};
function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const user: User = {details: {name: " Pat ", scores: [i, seed]}};
  const saved: User = {details: {name: user.details.name, scores: user.details.scores.slice()}};
  user.details.scores[0] = i + 1;
  sum += saved.details.scores[0] + user.details.scores[0] + saved.details.scores[1];
  sum += saved.details.scores.length;
 } return sum;
}
