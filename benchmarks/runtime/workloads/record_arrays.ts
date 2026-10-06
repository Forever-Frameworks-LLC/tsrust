type Profile = {score: number}; type Entry = {profile: Profile; name: string}; type Catalog = {users: Entry[]; rows: number[][]};
function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const catalog: Catalog = {users: [{profile: {score: i}, name: "Pat"}], rows: [[seed]]};
  append(catalog, i); for (const user of catalog.users) sum += user.profile.score;
  for (const row of catalog.rows) for (const score of row) sum += score;
  catalog.users[0].profile = {score: i + 1}; sum += catalog.users[0].profile.score;
 } return sum;
}
function append(catalog: Catalog, value: number): void { catalog.users.push({profile: {score: value + 2}, name: "Sam"}); catalog.rows[0].push(value); }
