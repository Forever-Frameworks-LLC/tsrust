type User = {name: string; value: number};
function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const user: User = {name: " Pat ", value: i};
  const read = (item: User): number => item.value; sum += readTwice(read, user);
  const mutate = (item: User): void => { item.value += seed; }; apply(mutate, user); sum += user.value;
  const take = (item: User): User => item; const saved = takeOnce(take, user); sum += saved.value;
 } return sum;
}
function readTwice(action: (user: User) => number, user: User): number { return action(user) + action(user); }
function apply(action: (user: User) => void, user: User): void { action(user); }
function takeOnce(action: (user: User) => User, user: User): User { return action(user); }
