type User = {name: string; value: number};
function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) { const action = make({name: "Pat", value: i}, seed);
  sum += action() + action(); const take = onceValue({name: "Pat", value: seed}); sum += take();
 } return sum;
}
function make(user: User, step: number): () => number {
 let saved = user;
 return () => { const previous = saved.value; saved = {name: "Sam", value: previous + step}; return previous + saved.value + 1; };
}
function onceValue(user: User): () => number { return () => { const moved = user; return moved.value; }; }
