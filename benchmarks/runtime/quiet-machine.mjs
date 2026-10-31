// Timing is only trustworthy on a quiet machine. Before each timed process the
// runner waits until the one-minute load average drops below a threshold that
// leaves room for the benchmark itself, and refuses to time if it never does.
import os from 'node:os';

// Half the logical CPUs: desktop apps stay below it, a parallel build does not.
export const DEFAULT_LIMIT = Math.max(2, os.cpus().length / 2);

export function quietEnough(load, limit = DEFAULT_LIMIT) {
  return Number.isFinite(load) && load < limit;
}

export function waitForQuietMachine({limit = DEFAULT_LIMIT, timeoutMs = 15 * 60_000, pollMs = 15_000, loadAverage = () => os.loadavg()[0], sleep = ms => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms), log = console.log} = {}) {
  const started = Date.now();
  let load = loadAverage();
  while (!quietEnough(load, limit)) {
    if (Date.now() - started >= timeoutMs) throw new Error(`Machine stayed busy (1-minute load ${load.toFixed(2)} >= ${limit}); stop other work before timing, or pass --allow-busy for an untrusted run.`);
    log(`Waiting for a quiet machine: 1-minute load ${load.toFixed(2)} >= ${limit}`);
    sleep(pollMs);
    load = loadAverage();
  }
  return load;
}
