async function step(value: number): Promise<number> {
  await Promise.resolve();
  return value * 2 + 1;
}
async function batch(): Promise<number> {
  let checksum = 0;
  for (let value = 0; value < 4096; value += 4) {
    const outcomes = await Promise.allSettled([
      step(value), step(value + 1), step(value + 2), step(value + 3),
    ]);
    for (const outcome of outcomes) {
      if (outcome.status === 'rejected') throw outcome.reason;
      checksum += outcome.value;
    }
  }
  return checksum;
}
async function main(): Promise<void> {
  for (let warmup = 0; warmup < 8; warmup++) await batch();
  for (let sample = 0; sample < 31; sample++) {
    const start = performance.now();
    const checksum = await batch();
    console.log(checksum);
    console.log(performance.now() - start);
  }
}
void main();
