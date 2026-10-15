// Explicit thunk matches Tsrust's lazy construction; Promise execution alone is eager.
type Task<T> = () => Promise<T>;
function step(value: number): Task<number> {
  return async () => value * 2 + 1;
}
function batch(): Task<number> {
  return async () => {
    let checksum = 0;
    for (let value = 0; value < 4096; value++) {
      const task = step(value);
      checksum += await task();
    }
    return checksum;
  };
}
async function main(): Promise<void> {
  for (let warmup = 0; warmup < 8; warmup++) await batch()();
  for (let sample = 0; sample < 31; sample++) {
    const start = performance.now();
    const checksum = await batch()();
    console.log(checksum);
    console.log(performance.now() - start);
  }
}
void main();
