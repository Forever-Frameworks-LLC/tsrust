// Independent integer transition oracle; deterministic destruction is native-only.
function counter() {
  let value = 0n;
  let live = true;
  return {
    add(delta: bigint, disabled = false): boolean {
      if (!live || disabled) return false;
      const next = value + delta;
      if (next < -(1n << 63n) || next > (1n << 63n) - 1n) return false;
      value = next;
      return true;
    },
    read(): bigint | undefined { return live ? value : undefined; },
    dispose() { live = false; },
  };
}
const first = counter();
const second = counter();
const observations: string[] = [];
function observe() { observations.push(`${first.read()},${second.read()}`); }
observe();
first.add(1n); observe();
first.add(100n, true); observe();
second.add(1n); observe();
second.add(1n); observe();
if (observations.join('|') !== '0,0|1,0|1,0|1,1|1,2') throw Error('Counter oracle failed');
first.dispose(); second.dispose();
if (first.add(1n) || first.read() !== undefined) throw Error('Disposed oracle accepted an update');
