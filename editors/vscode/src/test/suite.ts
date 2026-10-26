import * as assert from 'node:assert/strict';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { testModules } from './modules';
import { testAgreement } from './agreement';
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
async function waitFor(predicate: () => boolean, label: string): Promise<void> {
  const start = Date.now();
  while (!predicate()) {
    assert.ok(Date.now() - start < 10000, `Timed out: ${label}`);
    await delay(25);
  }
}
async function replace(editor: vscode.TextEditor, text: string): Promise<void> {
  const doc = editor.document;
  assert.ok(await editor.edit(edit => edit.replace(new vscode.Range(doc.positionAt(0), doc.positionAt(doc.getText().length)), text)));
}
export async function run(): Promise<void> {
  const folder = vscode.workspace.workspaceFolders![0].uri;
  const extension = vscode.extensions.getExtension('tsrust-local.tsrust');
  assert.ok(extension);
  await extension.activate();
  const uri = vscode.Uri.joinPath(folder, 'live.tsr');
  const ownership = 'type User = { name: string; };\nfunction take(user: User): User { return user; }\nfunction main() {\n const user: User = { name: "😀" };\n const saved = take(user);\n console.log(saved.name);\n console.log(user.name);\n}';
  await vscode.workspace.fs.writeFile(uri, Buffer.from('function main() {}'));
  const document = await vscode.workspace.openTextDocument(uri);
  let editor = await vscode.window.showTextDocument(document, { preview: false });
  assert.equal(document.languageId, 'tsrust');
  await replace(editor, ownership);
  assert.ok(document.isDirty, 'diagnostics must check unsaved contents');
  await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.source === 'tsrust' && !!d.relatedInformation?.length), 'unsaved ownership diagnostic with related transfer');
  const ownershipError = vscode.languages.getDiagnostics(uri)[0];
  assert.equal(ownershipError.severity, vscode.DiagnosticSeverity.Error);
  assert.ok(document.getText(ownershipError.range).includes('user'));
  await replace(editor, 'function main() {}');
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'repair clears errors');
  await replace(editor, 'function main(){const text="😀" /* keep */\nconsole.log(text);}');
  await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.code === 'syntax'), 'quick fix diagnostic');
  const fixDiagnostic = vscode.languages.getDiagnostics(uri)[0];
  const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>('vscode.executeCodeActionProvider', uri, fixDiagnostic.range, vscode.CodeActionKind.QuickFix.value);
  const fix = actions?.find(action => action.title === 'Insert missing ;');
  assert.ok(fix?.edit, 'compiler-backed missing semicolon quick fix');
  assert.ok(await vscode.workspace.applyEdit(fix.edit));
  assert.ok(document.getText().includes('"😀"; /* keep */'), 'fix preserves comments and Unicode');
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'quick fix clears diagnostic');
  await replace(editor, ownership.replace("const saved = take(user);", "const user2 = user;\n const saved = take(user);"));
  await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.code === 'transferred'), 'clone diagnostic');
  const cloneDiagnostic = vscode.languages.getDiagnostics(uri).find(d => d.code === 'transferred')!;
  const cloneActions = await vscode.commands.executeCommand<vscode.CodeAction[]>('vscode.executeCodeActionProvider', uri, cloneDiagnostic.range, vscode.CodeActionKind.QuickFix.value);
  const cloneFix=cloneActions?.find(action => action.title.startsWith('Clone before'));
  assert.ok(cloneFix?.edit, 'verified clone action at earlier transfer');
  assert.ok(await vscode.workspace.applyEdit(cloneFix.edit));
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'clone clears ownership diagnostic');
  await replace(editor, 'function main(){console.log("😀");}');
  await vscode.commands.executeCommand('tsrust.formatDocument');
  assert.ok(document.getText().includes('\n  console.log'), 'format command expands block with two-space indentation');
  assert.equal(await vscode.commands.executeCommand('tsrust.checkFile',uri),0, 'file checks from command');
  const taskEnd = new Promise<number | undefined>(resolve => {
    const subscription=vscode.tasks.onDidEndTaskProcess(event => { if(event.execution.task.source === 'Tsrust') {subscription.dispose();resolve(event.exitCode);} });
  });
  const task=await vscode.commands.executeCommand<vscode.TaskExecution>('tsrust.runFile',uri);
  assert.ok(task, 'play/run command starts process task');
  assert.equal(await Promise.race([taskEnd,delay(60000).then(()=>{throw new Error('Run task timed out');})]),0);
  const configuration=vscode.workspace.getConfiguration('tsrust',uri);
  await configuration.update('formatOnSave',true,vscode.ConfigurationTarget.Workspace);
  await replace(editor,'function main(){console.log(8);}');
  assert.ok(await document.save());
  assert.ok(document.getText().includes('\n  console.log'), 'format-on-save applies two-space indentation before writing');
  await configuration.update('formatOnSave',false,vscode.ConfigurationTarget.Workspace);
  const unicode = 'function main() { const text = "😀é"; const x: int = true; }';
  await replace(editor, unicode);
  await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.code === 'type'), 'Unicode type diagnostic');
  const diagnostic = vscode.languages.getDiagnostics(uri)[0];
  assert.equal(document.getText(diagnostic.range), 'true');
  assert.equal(diagnostic.range.start.character, unicode.indexOf('true'));
  await replace(editor, 'function main( {');
  await waitFor(() => vscode.languages.getDiagnostics(uri).some(d => d.code === 'syntax'), 'incomplete syntax');
  const secondUri = vscode.Uri.joinPath(folder, 'second.tsr');
  await vscode.workspace.fs.writeFile(secondUri, Buffer.from('function main() { const x: int = false; }'));
  const second = await vscode.workspace.openTextDocument(secondUri);
  await vscode.window.showTextDocument(second);
  await waitFor(() => vscode.languages.getDiagnostics(secondUri).length > 0, 'second file checked independently');
  editor = await vscode.window.showTextDocument(document, { preview: false });
  for (let i = 0; i < 5; i++) await replace(editor, i === 4 ? 'function main() {}' : 'function main( {');
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'rapid edits use latest version');
  await vscode.commands.executeCommand('tsrust.restartServer');
  await waitFor(() => vscode.languages.getDiagnostics(secondUri).length > 0, 'restart republishes open document');
  const intelligence = `/** Adds two values.
 * @param left - The first value.
 * @param right - The second value.
 * @returns The sum.
 */
function sum(left: int, right: int): int { return left + right; }
type User = { name: string; age: int; };
function main() { const user: User = { name: "Pat", age: 2 }; const result = sum(1, 2); user.name; }
`;
  await replace(editor, intelligence);
  const callOffset = document.getText().lastIndexOf('sum(');
  const hovers = await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider', uri, document.positionAt(callOffset + 1));
  assert.ok(hovers?.some(hover => hover.contents.some(content => typeof content !== 'string' && 'value' in content && content.value.includes('Adds two values.'))), 'function hover includes documentation');
  const help = await vscode.commands.executeCommand<vscode.SignatureHelp>('vscode.executeSignatureHelpProvider', uri, document.positionAt(callOffset + 7), '(', false);
  assert.ok(help?.signatures[0].label.includes('left: int, right: int'));
  assert.equal(help.activeParameter, 1);
  const definitions = await vscode.commands.executeCommand<vscode.Location[]>('vscode.executeDefinitionProvider', uri, document.positionAt(callOffset + 1));
  assert.ok(definitions?.length);
  const symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>('vscode.executeDocumentSymbolProvider', uri);
  assert.ok(symbols?.some(symbol => symbol.name === 'sum'));
  const references = await vscode.commands.executeCommand<vscode.Location[]>('vscode.executeReferenceProvider', uri, document.positionAt(callOffset + 1));
  assert.equal(references?.length, 2);
  await replace(editor, intelligence.replace('user.name;', 'user.na'));
  const memberPosition = document.positionAt(document.getText().lastIndexOf('user.na') + 7);
  const members = await vscode.commands.executeCommand<vscode.CompletionList>('vscode.executeCompletionItemProvider', uri, memberPosition, '.');
  assert.ok(members?.items.some(item => item.label === 'name'), 'record-field autocomplete works in incomplete source');
  await replace(editor, intelligence.replace('user.name;', 'su'));
  const completionPosition = document.positionAt(document.getText().lastIndexOf('su') + 2);
  const completions = await vscode.commands.executeCommand<vscode.CompletionList>('vscode.executeCompletionItemProvider', uri, completionPosition);
  assert.ok(completions?.items.some(item => item.label === 'sum'), 'function autocomplete works in incomplete source');
  const extensionManifest = extension.packageJSON;
  assert.ok(extensionManifest.contributes.languages[0].icon.dark.endsWith('tsrust.svg'));
  await replace(editor, 'function main() {}');
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'repair after intelligence tests');
  await document.save();
  await vscode.commands.executeCommand('workbench.action.closeActiveEditor');
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'close clears errors');
  const reopened = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(reopened);
  await waitFor(() => vscode.languages.getDiagnostics(uri).length === 0, 'reopened document diagnostics settle');
  assert.equal(vscode.languages.getDiagnostics(uri).length, 0, JSON.stringify(vscode.languages.getDiagnostics(uri)));
  await testModules(folder);
  await testAgreement(folder);
  // Missing executable must leave language recognition and lexical editing available.
  await vscode.workspace.getConfiguration('tsrust').update('server.path', path.join(folder.fsPath, 'missing-server'), vscode.ConfigurationTarget.Workspace);
  await vscode.commands.executeCommand('tsrust.restartServer');
  assert.equal(reopened.languageId, 'tsrust');
  console.log('Tsrust extension E2E: unsaved ownership/type/syntax errors, UTF-16, multiple documents, repair, rapid edits, restart, close/reopen, missing server, documentation hover, signatures, autocomplete, definitions, references, outline and icon contribution passed.');
}
