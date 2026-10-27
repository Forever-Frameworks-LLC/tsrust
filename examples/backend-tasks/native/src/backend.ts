import { spawn } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { createClient } from '@tsrust-example/tasks-api';
import { createHttpTransport } from '@tsrust-example/tasks-api/http';

export function deadline<T>(promise: Promise<T>, milliseconds: number, message: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  return Promise.race([promise, new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(message)), milliseconds);
  })]).finally(() => clearTimeout(timer));
}

export async function startBackend(configuration: {
  executable: string; database: string; migrations: string; startupTimeoutMs?: number;
}) {
  const token = randomBytes(32).toString('hex');
  const databaseUrl = pathToFileURL(configuration.database);
  if (databaseUrl.host) throw new Error('Desktop data must use a local filesystem');
  const databasePath = process.platform === 'win32' ? databaseUrl.pathname.slice(1) : databaseUrl.pathname;
  const child = spawn(configuration.executable, [], {
    windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'],
    env: { ...process.env, TASK_DESKTOP_TOKEN: token,
      TASK_DESKTOP_DATABASE_URL: `sqlite:${databasePath}?mode=rwc`,
      TASK_DESKTOP_MIGRATIONS: configuration.migrations },
  });
  child.stdin.on('error', () => {});
  child.stderr.resume();
  const exited = new Promise<{ code: number | null; signal: NodeJS.Signals | null }>(resolve => {
    child.once('close', (code, signal) => resolve({ code, signal }));
  });
  let address: string | undefined, buffer = '';
  const ready = new Promise<string>((resolve, reject) => {
    child.once('error', () => reject(new Error('The native backend could not be launched')));
    child.once('exit', () => reject(new Error('The native backend exited before readiness')));
    child.stdout.on('data', (chunk: Buffer) => {
      if (address) return;
      if (buffer.length + chunk.length > 8192) { reject(new Error('Invalid native backend readiness')); return; }
      buffer += chunk.toString('utf8');
      const end = buffer.indexOf('\n'); if (end < 0) return;
      try {
        const value = JSON.parse(buffer.slice(0, end));
        const match = typeof value.address === 'string' && /^127\.0\.0\.1:([1-9]\d{0,4})$/.exec(value.address);
        if (value.kind !== 'ready' || value.protocol !== 'http' || !match || Number(match[1]) > 65535) throw new Error();
        address = `http://${value.address}`; resolve(address);
      } catch { reject(new Error('Invalid native backend readiness')); }
    });
  });
  async function terminate() {
    if (child.exitCode === null && child.signalCode === null) child.stdin.end();
    try { return await deadline(exited, 2000, 'Native backend did not exit after parent-pipe closure'); }
    catch { child.kill(); return deadline(exited, 3000, 'Native backend could not be terminated'); }
  }
  try { address = await deadline(ready, configuration.startupTimeoutMs ?? 20000, 'Native backend readiness timed out'); }
  catch (error) { await terminate(); throw error; }
  const headers = { authorization: `Bearer ${token}` };
  const api = createClient(createHttpTransport({ url: address + '/api', headers,
    fetch: (input, init) => fetch(input, { ...init, signal: AbortSignal.timeout(6000) }),
  }));
  let stopping: Promise<void> | undefined;
  return {
    api, pid: child.pid!, exited,
    stop() {
      return stopping ??= (async () => {
        if (child.exitCode !== null || child.signalCode !== null) { await exited; return; }
        try {
          const response = await fetch(address + '/__desktop_shutdown', { method: 'POST', headers, signal: AbortSignal.timeout(6000) });
          if (!response.ok) throw new Error('Native shutdown was rejected');
          await response.text();
          const result = await deadline(exited, 8000, 'Native backend drain timed out');
          if (result.code !== 0) throw new Error('Native backend did not exit cleanly');
        } catch (error) { await terminate(); throw error; }
      })();
    },
  };
}
