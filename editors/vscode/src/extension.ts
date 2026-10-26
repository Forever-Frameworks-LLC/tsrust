import { dirname } from 'node:path';
import { resolveExecutable } from './executables';
import { execFile } from 'node:child_process';
import * as vscode from 'vscode';
import { LanguageClient, State, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let lifecycle = Promise.resolve();
let disposed = false;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  disposed = false;
  const output = vscode.window.createOutputChannel('Tsrust');
  const sourceWatcher = vscode.workspace.createFileSystemWatcher('**/*.{tsr,tsrx}');
  const dataWatcher = vscode.workspace.createFileSystemWatcher('**/*.{json,jsonc,rs,lock}');
  context.subscriptions.push(sourceWatcher, dataWatcher);
  const status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);
  status.command = 'tsrust.restartServer';
  status.tooltip = 'Tsrust compiler diagnostics. Click to restart the language server.';
  const showStatus = () => {
    if (vscode.window.activeTextEditor?.document.languageId === 'tsrust') status.show();
    else status.hide();
  };
  context.subscriptions.push(output, status, vscode.window.onDidChangeActiveTextEditor(showStatus));
  const compilerPath = async (uri: vscode.Uri): Promise<string> => {
    if (!vscode.workspace.isTrusted) throw new Error('Trust this workspace before running Tsrust commands.');
    const settings = vscode.workspace.getConfiguration('tsrust', uri);
    const configured = settings.get<string>('compiler.path', '').trim();
    const server = settings.get<string>('server.path', '').trim();
    return resolveExecutable('tsrust', {configured, companion: server});
  };
  const activeFile = async (uri?: vscode.Uri): Promise<vscode.TextDocument> => {
    if (!vscode.workspace.isTrusted) throw new Error('Trust this workspace before running Tsrust commands.');
    const document = uri ? await vscode.workspace.openTextDocument(uri) : vscode.window.activeTextEditor?.document;
    if (!document || document.languageId !== 'tsrust' || document.uri.scheme !== 'file') throw new Error('Open a saved .tsr or .tsrx file first.');
    if (document.isDirty && !await document.save()) throw new Error('Save was cancelled; the file was not run.');
    return document;
  };
  const runFile = async (uri?: vscode.Uri) => {
    try {
      const document = await activeFile(uri);
      const compiler = await compilerPath(document.uri);
      const settings = vscode.workspace.getConfiguration('tsrust', document.uri);
      const args = ['run', document.uri.fsPath];
      if (settings.get<boolean>('run.release', false)) args.push('--release');
      const folder = vscode.workspace.getWorkspaceFolder(document.uri);
      const task = new vscode.Task({type:'tsrust', file:document.uri.fsPath}, folder || vscode.TaskScope.Workspace,
        `Run ${document.uri.path.split('/').pop()}`, 'Tsrust', new vscode.ProcessExecution(compiler, args, {cwd:dirname(document.uri.fsPath)}));
      task.presentationOptions = {reveal:vscode.TaskRevealKind.Always, panel:vscode.TaskPanelKind.Dedicated, clear:true, echo:false, showReuseMessage:false,
        close:settings.get<boolean>('run.closeOnSuccess', false)};
      return await vscode.tasks.executeTask(task);
    } catch (error) { void vscode.window.showErrorMessage(`Tsrust: ${String(error)}`); }
  };
  const checkFile = async (uri?: vscode.Uri, automatic = false) => {
    try {
      const document = await activeFile(uri);
      const compiler = await compilerPath(document.uri);
      return await new Promise<number>(resolve => execFile(compiler, ['check', document.uri.fsPath], {cwd:dirname(document.uri.fsPath)}, (error, stdout, stderr) => {
        if (error || !automatic) {
          output.clear();
          output.appendLine((stdout + stderr).trim() || `${document.uri.path.split('/').pop()}: no errors.`);
          if (!automatic) output.show(true);
        }
        resolve(error ? 1 : 0);
      }));
    } catch (error) { void vscode.window.showErrorMessage(`Tsrust: ${String(error)}`); }
  };
  context.subscriptions.push(
    vscode.commands.registerCommand('tsrust.runFile', runFile),
    vscode.commands.registerCommand('tsrust.checkFile', checkFile),
    vscode.commands.registerCommand('tsrust.formatDocument', () => vscode.commands.executeCommand('editor.action.formatDocument')),
    vscode.workspace.onDidSaveTextDocument(document => {
      if (document.languageId === 'tsrust' && vscode.workspace.getConfiguration('tsrust', document.uri).get<boolean>('checkOnSave', false)) void checkFile(document.uri, true);
    }),
    vscode.workspace.onWillSaveTextDocument(event => {
      if (event.document.languageId !== 'tsrust' || !client || !vscode.workspace.isTrusted || !vscode.workspace.getConfiguration('tsrust', event.document.uri).get<boolean>('formatOnSave', false)) return;
      const document=event.document; const version=document.version;
      event.waitUntil(client.sendRequest<Array<{range:{start:{line:number;character:number};end:{line:number;character:number}};newText:string}>>('textDocument/formatting', {textDocument:{uri:document.uri.toString()},options:{tabSize:2,insertSpaces:true}}).then(edits => {
        if (document.version !== version) return [];
        return edits.map(edit => vscode.TextEdit.replace(new vscode.Range(edit.range.start.line, edit.range.start.character, edit.range.end.line, edit.range.end.character),edit.newText));
      }).catch(error => {output.appendLine(`Formatting: ${String(error)}`);return [];}));
    }),
  );
  let stateSubscription: vscode.Disposable | undefined;
  context.subscriptions.push({ dispose: () => stateSubscription?.dispose() });
  const start = async () => {
    stateSubscription?.dispose();
    stateSubscription = undefined;
    await client?.dispose();
    client = undefined;
    if (disposed) return;
    if (!vscode.workspace.isTrusted) {
      status.text = 'Tsrust: trust workspace to check';
      return;
    }
    const settings = vscode.workspace.getConfiguration('tsrust');
    let path: string;
    try {
      path = await resolveExecutable('tsrust-lsp', {configured: settings.get<string>('server.path', '').trim(), companion: settings.get<string>('compiler.path', '').trim()});
    } catch (error) {
      status.text = 'Tsrust: configure server';
      const message = `Tsrust analysis is unavailable. ${error instanceof Error ? error.message : String(error)} Highlighting remains available.`;
      output.appendLine(message);
      void vscode.window.showWarningMessage(message, 'Open Settings').then(choice => {
        if (choice) void vscode.commands.executeCommand('workbench.action.openSettings', 'tsrust.server.path');
      });
      return;
    }
    const next = new LanguageClient('tsrust', 'Tsrust', {
      command: path, args: [], transport: TransportKind.stdio,
    }, {
      documentSelector: [{ language: 'tsrust', scheme: 'file' }, { language: 'tsrust', scheme: 'untitled' }, { scheme: 'file', pattern: '**/*.json' }, { scheme: 'file', pattern: '**/*.jsonc' }, { scheme: 'file', pattern: '**/*.rs' }, { scheme: 'file', pattern: '**/*.bindings.lock' }],
      outputChannel: output,
      synchronize: { fileEvents: [sourceWatcher, dataWatcher] },
      initializationOptions: { debounceMs: settings.get<number>('analysis.debounceMs', 200) },
      // Use the client's bounded recovery and protocol-error handling. An
      // unexpected exit should not permanently disable editing after one failure.
      connectionOptions: { maxRestartCount: 4 },
    });
    client = next;
    stateSubscription = next.onDidChangeState(event => {
      if (client !== next) return;
      status.text = event.newState === State.Running ? 'Tsrust: checking enabled' : event.newState === State.Starting ? 'Tsrust: starting' : 'Tsrust: stopped';
    });
    try {
      await next.start();
      if (disposed) await next.dispose();
    } catch (error) {
      status.text = 'Tsrust: failed to start';
      output.appendLine(String(error));
      void vscode.window.showErrorMessage('Tsrust language server failed to start. See the Tsrust output channel, then run Tsrust: Restart Language Server.');
      await next.dispose();
      if (client === next) client = undefined;
    }
  };
  const restart = () => {
    lifecycle = lifecycle.then(start).catch(error => {
      output.appendLine(`Server lifecycle failure: ${String(error)}`);
      status.text = 'Tsrust: stopped';
    });
    return lifecycle;
  };
  context.subscriptions.push(
    vscode.commands.registerCommand('tsrust.restartServer', restart),
    vscode.workspace.onDidChangeConfiguration(event => {
      if (event.affectsConfiguration('tsrust.server.path') || event.affectsConfiguration('tsrust.compiler.path') || event.affectsConfiguration('tsrust.analysis.debounceMs')) void restart();
    }),
    vscode.workspace.onDidGrantWorkspaceTrust(() => { void restart(); }),
  );
  showStatus();
  await restart();
}

export async function deactivate(): Promise<void> {
  disposed = true;
  await lifecycle;
  await client?.dispose();
  client = undefined;
}
