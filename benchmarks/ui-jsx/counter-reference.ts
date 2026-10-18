// Independent integer transition oracle; deterministic destruction is native-only.
function counter() {
  let value = 0n;
  let live = true;
  return {
    get(): bigint { if (!live) throw Error('Disposed state'); return value; },
    set(next: bigint): void { if (!live) throw Error('Disposed state'); value = next; },
    dispose() { live = false; },
  };
}
function increment(state: ReturnType<typeof counter>, delta = 1n, disabled = false) {
  if (disabled) return;
  const next = state.get() + delta;
  if (next < -(1n << 63n) || next > (1n << 63n) - 1n) throw Error('Integer overflow');
  state.set(next);
}
const first = counter();
const second = counter();
const observations: string[] = [];
function observe() { observations.push(`${first.get()},${second.get()}`); }
observe();
increment(first); observe();
increment(first, 100n, true); observe();
increment(second); observe();
increment(second); observe();
if (observations.join('|') !== '0,0|1,0|1,0|1,1|1,2') throw Error('Counter oracle failed');
first.set((1n << 63n) - 1n);
let overflow = false;
try { increment(first); } catch { overflow = true; }
if (!overflow || first.get() !== (1n << 63n) - 1n) throw Error('Overflow mutated state');
first.dispose(); second.dispose();
let disposed = false;
try { increment(first); } catch { disposed = true; }
if (!disposed) throw Error('Disposed oracle accepted an update');
