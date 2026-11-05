import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {once} from 'node:events';

// Await lifecycle replies before sending the next message, especially exit.
export async function smokeLsp(command, args, {cwd, rootUri, timeout = 30000}) {
  const child = spawn(command, args, {cwd, windowsHide: true, stdio: ['pipe', 'pipe', 'inherit']});
  let failure, timedOut = false, pending = Buffer.alloc(0);
  child.on('error', error => { failure = error; });
  child.stdin.on('error', error => { failure = error; });
  const closed = new Promise(resolve => child.once('close', (code, signal) => resolve({code, signal})));
  const timer = setTimeout(() => { timedOut = true; child.kill('SIGKILL'); }, timeout);
  const chunks = child.stdout[Symbol.asyncIterator]();
  async function send(method, params, id) {
    const body = Buffer.from(JSON.stringify({jsonrpc: '2.0', method, params, ...(id === undefined ? {} : {id})}));
    const frame = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
    await new Promise((resolve, reject) => child.stdin.write(frame, error => error ? reject(error) : resolve()));
  }
  async function response(id) {
    while (true) {
      const boundary = pending.indexOf('\r\n\r\n');
      if (boundary >= 0) {
        const length = Number(/Content-Length: (\d+)/i.exec(pending.subarray(0, boundary).toString())?.[1]);
        assert.ok(Number.isSafeInteger(length) && length > 0, 'Invalid LSP frame length');
        if (pending.length >= boundary + 4 + length) {
          const value = JSON.parse(pending.subarray(boundary + 4, boundary + 4 + length));
          pending = pending.subarray(boundary + 4 + length);
          if (value.id === undefined) continue;
          assert.equal(value.id, id, 'Unexpected LSP response');
          assert.equal(value.error, undefined, JSON.stringify(value.error));
          return value;
        }
      }
      const chunk = await chunks.next();
      assert.ok(!timedOut, 'LSP lifecycle timed out');
      if (failure) throw failure;
      assert.ok(!chunk.done, `LSP closed before response ${id}`);
      pending = Buffer.concat([pending, chunk.value]);
    }
  }
  try {
    await once(child, 'spawn');
    await send('initialize', {processId: null, rootUri, capabilities: {}}, 1);
    assert.ok((await response(1)).result?.capabilities?.hoverProvider, 'LSP must advertise hover');
    await send('initialized', {});
    await send('shutdown', null, 2);
    assert.deepEqual(await response(2), {jsonrpc: '2.0', id: 2, result: null});
    await send('exit', null);
    child.stdin.end();
    assert.deepEqual(await closed, {code: 0, signal: null}, 'LSP must exit cleanly');
    assert.ok(!timedOut, 'LSP lifecycle timed out');
  } finally {
    if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
    await closed;
    clearTimeout(timer);
  }
}
