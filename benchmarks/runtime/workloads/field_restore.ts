type Details = {name: string; scores: number[]}; type User = {details: Details; score: number};
function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const user: User = {details: {name: "Pat", scores: [i, seed]}, score: i + 1};
  const scores = user.details.scores, name = user.details.name; sum += user.score;
  user.details.scores = [i + 2]; user.details.name = "Sam";
  for (const score of scores) sum += score;
  sum += user.details.scores[0]; const restored = user; sum += restored.score;
 } return sum;
}
