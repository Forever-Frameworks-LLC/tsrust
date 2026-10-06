type User = {name: string; value: number};
function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const first = counter(i), moved = identity(first), wrapped = retain(moved);
  sum += wrapped() + wrapped();
  const take = (user: User): User => user; const user = consume(take, {name: " Pat ", value: seed});
  sum += user.value;
 } return sum;
}
function counter(start: number): () => number { let value = start; return () => ++value; }
function identity(action: () => number): () => number { const moved = action; return moved; }
function retain(action: () => number): () => number { return () => action(); }
function consume(action: (user: User) => User, user: User): User { return action(user); }
