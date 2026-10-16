// Matches the successful/retry loading contract, not native cancellation or ownership.
import assert from 'node:assert/strict';
class Loader {
  readonly values = new Map<string, string>();
  private tail = Promise.resolve();
  loads = 0;
  private closed = false;
  private readonly finished = new Set<number>();
  finish(client: number): Promise<void> {
    const done = this.tail.then(() => {
      assert(!this.finished.has(client));
      this.finished.add(client);
      if (this.finished.size === 2) this.closed = true;
    });
    this.tail = done;
    return done;
  }
  get(key: string): Promise<string> {
    const result = this.tail.then(async () => {
      if (this.closed) throw new Error('closed');
      const cached = this.values.get(key);
      if (cached !== undefined) {
        this.values.delete(key);
        this.values.set(key, cached);
        return cached;
      }
      this.loads++;
      await new Promise(resolve => setTimeout(resolve, 1));
      if (key === 'failure') return 'failed';
      const value = 'loaded:' + key;
      if (this.values.size === 2) this.values.delete(this.values.keys().next().value!);
      this.values.set(key, value);
      return value;
    });
    this.tail = result.then(() => undefined, () => undefined);
    return result;
  }
}
async function main() {
  const loader = new Loader();
  async function client(id: number) {
    assert.equal(await loader.get('shared'), 'loaded:shared');
    assert.equal(await loader.get('shared'), 'loaded:shared');
    assert.equal(await loader.get('failure'), 'failed');
    await loader.finish(id);
  }
  await Promise.all([client(0), client(1)]);
  await assert.rejects(loader.get('late'), /closed/);
  assert.equal(loader.loads, 3); // One successful load plus two uncached failures.
  loader.values.clear();
  console.log('serialized cache loading accepted');
}
void main();
