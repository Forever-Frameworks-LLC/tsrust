// Bounded Windows/macOS diagnostic, not a benchmark or full-workspace acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawn, spawnSync} from 'node:child_process';
import {mkdtemp, writeFile, rm} from 'node:fs/promises';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';

assert.ok(['win32', 'darwin'].includes(process.platform), 'Supported counters: Windows and macOS.');
assert.ok(process.argv.slice(2).every(arg => arg === '--soak'), 'Usage: node tools/lsp-memory-probe.mjs [--soak]');
const soakMs = process.argv.includes('--soak') ? 30 * 60 * 1000 : 0;
const limitMs = soakMs ? soakMs + 60000 : 120000;
const target = path.resolve('target');
const root = await mkdtemp(path.join(target, 'lsp-memory-probe-'));
const server = path.resolve(process.env.TSRUST_LSP_SERVER || 'target/debug/tsrust-lsp' + (process.platform === 'win32' ? '.exe' : ''));
const child = spawn(server, [], {stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true});
const started = Date.now();
const samples = [];
const waiting = new Map();
const diagnostics = new Map();
let sequence = 0, pending = Buffer.alloc(0), stderr = '', failure;
child.stderr.on('data', value => { stderr += value; });
child.on('error', error => { failure = error; });
child.stdout.on('data', chunk => {
    pending = Buffer.concat([pending, chunk]);
    while (true) {
        const end = pending.indexOf('\r\n\r\n');
        if (end < 0) break;
        const length = Number(/Content-Length: (\d+)/i.exec(pending.subarray(0, end).toString())[1]);
        if (pending.length < end + 4 + length) break;
        const message = JSON.parse(pending.subarray(end + 4, end + 4 + length));
        pending = pending.subarray(end + 4 + length);
        if (message.id !== undefined) {
            waiting.get(message.id)?.(message);
            waiting.delete(message.id);
        }
        if (message.method === 'textDocument/publishDiagnostics') diagnostics.set(message.params.uri, message.params);
    }
});
function send(method, params, id) {
    const body = Buffer.from(JSON.stringify({jsonrpc: '2.0', method, params, ...(id === undefined ? {} : {id})}));
    child.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
    child.stdin.write(body);
}
async function request(method, params) {
    const id = ++sequence;
    const response = new Promise((resolve, reject) => {
        const timer = setTimeout(() => { waiting.delete(id); reject(new Error(`No response: ${method}`)); }, 10000);
        waiting.set(id, message => { clearTimeout(timer); resolve(message); });
    });
    send(method, params, id);
    const message = await response;
    assert.equal(message.error, undefined, JSON.stringify(message));
    return message.result;
}
function sample() {
    if (process.platform === 'darwin') {
        const output = path.join(root, 'footprint.json');
        const run = (command, args) => {
            const result = spawnSync(command, args, {encoding: 'utf8', timeout: 5000});
            assert.equal(result.status, 0, result.stderr || String(result.error));
            return result.stdout;
        };
        run('/usr/bin/footprint', ['--swapped', '--format', 'bytes', '--json', output, '-p', String(child.pid)]);
        const report = JSON.parse(readFileSync(output, 'utf8'));
        assert.equal(report.unit, 'byte');
        assert.deepEqual(report.errors, []);
        assert.equal(report.processes.length, 1);
        const processReport = report.processes[0];
        assert.equal(processReport.pid, child.pid);
        const pressure = run('/usr/bin/memory_pressure', ['-Q']);
        const free = /System-wide memory free percentage: (\d+)%/.exec(pressure);
        assert.ok(free, 'Missing system free-memory counter');
        const value = {
            elapsedMs: Date.now() - started, pendingRequests: waiting.size,
            physicalFootprint: processReport.footprint,
            physicalFootprintPeak: processReport.auxiliary.phys_footprint_peak,
            swappedCompressed: report.summary.total.swapped,
            residentBytes: Number(run('/bin/ps', ['-o', 'rss=', '-p', String(child.pid)]).trim()) * 1024,
            systemFreePercent: Number(free[1]),
        };
        for (const number of Object.values(value)) assert.ok(Number.isFinite(number) && number >= 0, 'Invalid process counter');
        samples.push(value);
        assert.ok(value.physicalFootprint < 1024 ** 3, 'Physical footprint limit exceeded');
        assert.ok(value.physicalFootprint - samples[0].physicalFootprint < 512 * 1024 ** 2, 'Physical footprint growth limit exceeded');
        assert.ok(value.swappedCompressed < 256 * 1024 ** 2, 'Swapped/compressed memory limit exceeded');
        assert.ok(value.systemFreePercent >= 20, 'System free-memory stop condition');
        return;
    }
    const result = spawnSync('powershell.exe', ['-NoProfile', '-Command',
        `$p=Get-Process -Id ${child.pid} -ErrorAction Stop; $os=Get-CimInstance Win32_OperatingSystem; [pscustomobject]@{workingSet=$p.WorkingSet64;privateBytes=$p.PrivateMemorySize64;availableBytes=($os.FreePhysicalMemory*1024)} | ConvertTo-Json -Compress`
    ], {encoding: 'utf8', windowsHide: true, timeout: 5000});
    assert.equal(result.status, 0, result.stderr);
    const value = {...JSON.parse(result.stdout), elapsedMs: Date.now() - started, pendingRequests: waiting.size};
    samples.push(value);
    assert.ok(value.privateBytes < 1024 ** 3, 'Private commit limit exceeded');
    assert.ok(value.workingSet - samples[0].workingSet < 512 * 1024 ** 2, 'Working set growth limit exceeded');
    assert.ok(value.availableBytes >= 4 * 1024 ** 3, 'Available physical memory stop condition');
}
const timeout = setTimeout(() => { failure = new Error(`${limitMs} ms probe limit`); child.kill(); }, limitMs);
const monitor = setInterval(() => {
    try { sample(); } catch (error) { failure = error; child.kill(); }
}, 2000);
const uri = name => pathToFileURL(path.join(root, name)).href;
try {
    await writeFile(path.join(root, 'library.tsr'), 'export function value():int{return 7;}\n');
    await request('initialize', {capabilities: {}, rootUri: uri(''), initializationOptions: {debounceMs: 0}});
    send('initialized', {});
    sample();
    const source = 'import {value} from "./library.tsr";\nfunction main(){const result=value();}\n';
    for (let cycle = 0; cycle < 12 || Date.now() - started < soakMs; cycle++) {
        await writeFile(path.join(root, 'library.tsr'), 'export function value():int{return 7;}\n');
        send('workspace/didChangeWatchedFiles', {changes: [{uri: uri('library.tsr'), type: 2}]});
        const opened = [];
        let lastHover;
        for (let index = 0; index < 24; index++) {
            const name = `entry${index}.tsr`;
            await writeFile(path.join(root, name), source);
            opened.push(name);
            send('textDocument/didOpen', {textDocument: {uri: uri(name), languageId: 'tsrust', version: 1, text: source}});
            const edits = await request('textDocument/formatting', {textDocument: {uri: uri(name)}, options: {tabSize: 4, insertSpaces: true}});
            const text = edits[0]?.newText || source;
            send('textDocument/didChange', {textDocument: {uri: uri(name), version: 2}, contentChanges: [{text}]});
            assert.deepEqual(await request('textDocument/formatting', {textDocument: {uri: uri(name)}, options: {tabSize: 4, insertSpaces: true}}), []);
            const line = text.split('\n').findIndex(value => value.includes('value();'));
            const character = text.split('\n')[line].indexOf('value();') + 2;
            lastHover = {textDocument: {uri: uri(name)}, position: {line, character}};
            const hover = await request('textDocument/hover', lastHover);
            assert.ok(JSON.stringify(hover).includes('int'), JSON.stringify(hover));
            // Distinct large unrelated revisions expose snapshots retaining all open buffers.
            const unrelated = uri('unrelated.tsr');
            const large = `// revision ${cycle}:${index} ${'x'.repeat(512 * 1024)}\nfunction unrelated() {}\n`;
            send(index ? 'textDocument/didChange' : 'textDocument/didOpen', index
                ? {textDocument: {uri: unrelated, version: index + 1}, contentChanges: [{text: large}]}
                : {textDocument: {uri: unrelated, languageId: 'tsrust', version: 1, text: large}});
            if (failure) throw failure;
        }
        await writeFile(path.join(root, 'library.tsr'), 'export function value():string{return "changed";}\n');
        send('workspace/didChangeWatchedFiles', {changes: [{uri: uri('library.tsr'), type: 2}]});
        assert.ok(JSON.stringify(await request('textDocument/hover', lastHover)).includes('string'), 'Dependency edit must invalidate imported help');
        const counters = await request('tsrust/counters', null);
        for (const name of ['staleAnswers', 'emptyKnownReceiver', 'nonMemberAfterSeparator']) assert.equal(counters[name], 0, JSON.stringify(counters));
        for (const name of [...opened, 'unrelated.tsr']) send('textDocument/didClose', {textDocument: {uri: uri(name)}});
        await new Promise(resolve => setTimeout(resolve, 2000));
        sample();
        if (failure) throw failure;
    }
    await request('shutdown', null);
    send('exit', null);
    await new Promise(resolve => child.once('exit', resolve));
    assert.equal(child.exitCode, 0, stderr);
    if (failure) throw failure;
} catch (error) {
    failure = error;
} finally {
    clearInterval(monitor);
    clearTimeout(timeout);
    child.kill();
    const evidence = {server, serverSha256: createHash('sha256').update(readFileSync(server)).digest('hex'), platform: process.platform, soakMs, elapsedMs: Date.now() - started, requests: sequence, samples, diagnosticDocuments: diagnostics.size, stderr, error: failure?.stack || null};
    await writeFile(path.join(target, 'lsp-memory-probe.json'), JSON.stringify(evidence, null, 2));
    assert.ok(root.startsWith(target + path.sep) && path.basename(root).startsWith('lsp-memory-probe-'));
    await rm(root, {recursive: true, force: true});
}
if (failure) throw failure;
console.log(JSON.stringify({requests: sequence, samples: samples.length, elapsedMs: Date.now() - started}));
