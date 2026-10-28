import * as assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import * as vscode from 'vscode';

const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

function serverPids(): number[] {
  // Only children of this isolated extension host are eligible. Never terminate
  // a server belonging to a user's normal editor or another workspace.
  if (process.platform === 'win32') {
    const script = `Get-CimInstance Win32_Process -Filter "ParentProcessId = ${process.pid}" | Where-Object { $_.Name -eq 'tsrust-lsp.exe' } | Select-Object -ExpandProperty ProcessId`;
    return execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], { encoding: 'utf8', windowsHide: true })
      .trim().split(/\s+/).filter(Boolean).map(Number);
  }
  const rows = execFileSync('ps', ['-ax', '-o', 'pid=,ppid=,comm='], { encoding: 'utf8' });
  return rows.split('\n').flatMap(row => {
    const fields = row.trim().split(/\s+/);
    return Number(fields[1]) === process.pid && fields.slice(2).join(' ').endsWith('tsrust-lsp') ? [Number(fields[0])] : [];
  });
}

async function waitFor(predicate: () => boolean, label: string): Promise<void> {
  const start = Date.now();
  while (!predicate()) {
    assert.ok(Date.now() - start < 15000, `Timed out: ${label}`);
    await delay(100);
  }
}

async function replace(editor: vscode.TextEditor, text: string): Promise<void> {
  const document = editor.document;
  assert.ok(await editor.edit(edit => edit.replace(new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), text)));
}

export async function run(): Promise<void> {
  const folder = vscode.workspace.workspaceFolders![0].uri;
  const uri = vscode.Uri.joinPath(folder, 'recovery.tsr');
  await vscode.workspace.fs.writeFile(uri, Buffer.from('function main() { const value: int = "invalid"; }'));
  const document = await vscode.workspace.openTextDocument(uri);
  const editor = await vscode.window.showTextDocument(document, { preview: false });
  await vscode.extensions.getExtension('tsrust-local.tsrust')!.activate();
  await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.source === 'tsrust'), 'initial diagnostics');

  for (let attempt = 0; attempt < 4; attempt++) {
    const pids = serverPids();
    assert.equal(pids.length, 1, 'exactly one server belongs to the isolated host');
    process.kill(pids[0]);
    await waitFor(() => {
      const next = serverPids();
      return next.length === 1 && next[0] !== pids[0];
    }, `automatic recovery ${attempt + 1}`);
    await replace(editor, `function main() { const value: int = ${attempt}; }`);
    await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'valid current text after recovery');
    await replace(editor, 'function main() { const value: int = "invalid"; }');
    await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.source === 'tsrust'), 'invalid current text after recovery');
  }

  const last = serverPids();
  assert.equal(last.length, 1);
  process.kill(last[0]);
  await waitFor(() => serverPids().length === 0, 'fifth exit stops restart loop');
  await delay(1500);
  assert.equal(serverPids().length, 0, 'restart budget remains exhausted');

  await vscode.commands.executeCommand('tsrust.restartServer');
  await waitFor(() => serverPids().length === 1, 'manual restart resets recovery budget');
  await replace(editor, 'function main() {}');
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'current document repair after restart');
  await replace(editor, 'function main() { const value: int = "invalid"; }');
  await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.source === 'tsrust'), 'current-text checking after manual restart');
  console.log('Tsrust recovery: four automatic restarts, bounded fifth exit, manual recovery and current-text diagnostics passed.');
}
