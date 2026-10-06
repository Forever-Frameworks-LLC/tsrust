type Counter = { value: number; active: boolean };
function workload(n: number, seed: number): number {
 const counter: Counter = {value: seed, active: true}; let sum = 0;
 for (let i = 0; i < n; i++) { increment(counter); if (counter.active) sum += counter.value; }
 return sum;
}
function increment(counter: Counter): void { counter.value += 1; counter.active = counter.value % 2 === 0; }
