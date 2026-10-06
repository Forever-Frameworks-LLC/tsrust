function workload(n: number, seed: number): number {
 let sum = 0;
 for (let i = 0; i < n; i++) {
  const action = (value: number, flag: boolean): number => flag ? value + seed : value;
  sum += forward(action, i); const onceAction = (): number => seed; sum += single(onceAction);
 } return sum;
}
function forward(action: (value: number, flag: boolean) => number, value: number): number { return action(value, true); }
function single(action: () => number): number { return action(); }
