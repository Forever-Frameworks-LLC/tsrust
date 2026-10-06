import * as assert from 'node:assert/strict';
import * as vscode from 'vscode';
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
async function eventually(check: () => Promise<boolean> | boolean, label: string): Promise<void> {
  const deadline = Date.now() + 10000;
  while (!await check()) {
    assert.ok(Date.now() < deadline, `Timed out: ${label}`);
    await delay(25);
  }
}
async function replace(editor: vscode.TextEditor, text: string): Promise<void> {
  const document = editor.document;
  assert.ok(await editor.edit(edit => edit.replace(new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), text)));
}
export async function testModules(folder: vscode.Uri): Promise<void> {
  const entryUri = vscode.Uri.joinPath(folder, 'module-main.tsr');
  const dependencyUri = vscode.Uri.joinPath(folder, 'module-dep.tsr');
  const library = '/** Adds one to the input. */\nexport function value(n: int): int { return n + 1; }';
  const entry = 'import { value as increment } from "./module-dep.tsr";\n\nfunction main() { console.log(increment(2)); }';
  await vscode.workspace.fs.writeFile(dependencyUri, Buffer.from(library));
  await vscode.workspace.fs.writeFile(entryUri, Buffer.from(entry));
  const dependency = await vscode.workspace.openTextDocument(dependencyUri);
  let dependencyEditor = await vscode.window.showTextDocument(dependency, { preview: false });
  const document = await vscode.workspace.openTextDocument(entryUri);
  let editor = await vscode.window.showTextDocument(document, { preview: false });
  const callPosition = document.positionAt(entry.lastIndexOf('increment(') + 1);
  await eventually(async () => {
    const hovers = await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider', entryUri, callPosition);
    return !!hovers?.some(hover => hover.contents.some(content => typeof content !== 'string' && 'value' in content && content.value.includes('Adds one to the input.')));
  }, 'imported alias hover from installed extension');
  const definitions = await vscode.commands.executeCommand<(vscode.Location | vscode.LocationLink)[]>('vscode.executeDefinitionProvider', entryUri, callPosition);
  assert.ok(definitions?.some(location => ('uri' in location ? location.uri : location.targetUri).toString() === dependencyUri.toString()));
  const references = await vscode.commands.executeCommand<vscode.Location[]>('vscode.executeReferenceProvider', entryUri, callPosition);
  assert.ok(references?.some(location => location.uri.toString() === dependencyUri.toString()));
  assert.ok(references?.some(location => location.uri.toString() === entryUri.toString()));
  dependencyEditor = await vscode.window.showTextDocument(dependency, { preview: false });
  await replace(dependencyEditor, 'export function value(n: int): int { const result=n+1 return result; }');
  assert.ok(dependency.isDirty);
  await eventually(() => vscode.languages.getDiagnostics(dependencyUri).some(d => d.code === 'syntax'), 'unsaved imported dependency diagnostic');
  let repair: vscode.CodeAction | undefined;
  await eventually(async () => {
    const diagnostic = vscode.languages.getDiagnostics(dependencyUri).find(d => d.code === 'syntax');
    if (!diagnostic) return false;
    const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>('vscode.executeCodeActionProvider', dependencyUri, diagnostic.range, vscode.CodeActionKind.QuickFix.value);
    repair = actions?.find(action => action.title === 'Insert missing ;');
    return !!repair?.edit;
  }, 'whole-project dependency quick fix');
  assert.ok(await vscode.workspace.applyEdit(repair!.edit!));
  await eventually(() => vscode.languages.getDiagnostics(dependencyUri).length === 0, 'dependency quick fix clears errors');
  assert.ok(await dependency.save());
  editor = await vscode.window.showTextDocument(document, { preview: false });
  await replace(editor, 'import {\n value as increment\n}\n\nfrom "./module-dep.tsr";\n\nfunction main(){console.log(increment(2));}');
  await vscode.commands.executeCommand('tsrust.formatDocument');
  assert.ok(document.getText().includes('}\u0020from "./module-dep.tsr";'), document.getText());
  assert.ok(!document.getText().includes('}\n\nfrom'));
  assert.ok(await document.save());
  assert.equal(await vscode.commands.executeCommand('tsrust.checkFile', entryUri), 0);
  const taskEnd = new Promise<number | undefined>(resolve => {
    const subscription = vscode.tasks.onDidEndTaskProcess(event => {
      if (event.execution.task.source === 'Tsrust') { subscription.dispose(); resolve(event.exitCode); }
    });
  });
  assert.ok(await vscode.commands.executeCommand('tsrust.runFile', entryUri));
  assert.equal(await Promise.race([taskEnd, delay(60000).then(() => { throw new Error('Module run task timed out'); })]), 0);
  dependencyEditor = await vscode.window.showTextDocument(dependency, { preview: false });
  await replace(dependencyEditor, 'import { basename } from "std:path";\nexport function factory(): Function<string, [path: ReadOnly<string>]> { return basename; }');
  assert.ok(await dependency.save());
  editor = await vscode.window.showTextDocument(document, { preview: false });
  const stdSource = 'import { basename as fileName } from "std:path";\nimport { factory as create } from "./module-dep.tsr";\nfunction main(){console.log(fileName("/tmp/report.txt", ".txt"));const stored=fileName;console.log(stored("/tmp/report.txt", ".txt"));const returned=create();console.log(returned("/tmp/factory.txt").toUpperCase());const selected=(true?fileName:stored);console.log(selected("/tmp/selected.txt", ".txt"));}';
  await replace(editor, stdSource);
  const stdPosition=document.positionAt(stdSource.lastIndexOf('fileName(')+1);
  await eventually(async () => {
    const hovers=await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider',entryUri,stdPosition);
    return !!hovers?.some(hover=>hover.contents.some(content=>typeof content!=='string' && 'value' in content && content.value.includes('suffix?: string') && content.value.includes('without reading the filesystem')));
  },'std alias documentation hover');
  const stdHelp=await vscode.commands.executeCommand<vscode.SignatureHelp>('vscode.executeSignatureHelpProvider',entryUri,document.positionAt(stdSource.indexOf('".txt"')));
  assert.equal(stdHelp?.activeParameter,1);
  assert.equal(stdHelp?.signatures[0].parameters?.[1].label,'suffix?: string');
  const storedPosition=document.positionAt(stdSource.lastIndexOf('stored(')+1);
  const storedHover=await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider',entryUri,storedPosition);
  assert.ok(storedHover?.some(hover=>hover.contents.some(content=>typeof content!=='string' && 'value' in content && content.value.includes('const stored: Function<string, [path: string, suffix?: string]>') && content.value.includes('without reading the filesystem'))));
  const storedHelp=await vscode.commands.executeCommand<vscode.SignatureHelp>('vscode.executeSignatureHelpProvider',entryUri,document.positionAt(stdSource.indexOf('".txt"',stdSource.lastIndexOf('stored('))));
  assert.equal(storedHelp?.activeParameter,1);
  assert.equal(storedHelp?.signatures[0].parameters?.[1].label,'suffix?: string');
  const returnedPosition=document.positionAt(stdSource.lastIndexOf('returned(')+1);
  await eventually(async () => {
    const hovers=await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider',entryUri,returnedPosition);
    return !!hovers?.some(hover=>hover.contents.some(content=>typeof content!=='string' && 'value' in content && content.value.includes('const returned: Function<string, [path: ReadOnly<string>]>')));
  },'imported factory result callable hover');
  const returnedHelp=await vscode.commands.executeCommand<vscode.SignatureHelp>('vscode.executeSignatureHelpProvider',entryUri,document.positionAt(stdSource.indexOf('"/tmp/factory.txt"')));
  assert.equal(returnedHelp?.signatures[0].parameters?.length,1);
  assert.equal(returnedHelp?.signatures[0].parameters?.[0].label,'path: ReadOnly<string>');
  const selectedPosition=document.positionAt(stdSource.lastIndexOf('selected(')+1);
  const selectedHover=await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider',entryUri,selectedPosition);
  assert.ok(selectedHover?.some(hover=>hover.contents.some(content=>typeof content!=='string' && 'value' in content && content.value.includes('suffix?: string') && content.value.includes('without reading the filesystem'))));
  const selectedHelp=await vscode.commands.executeCommand<vscode.SignatureHelp>('vscode.executeSignatureHelpProvider',entryUri,document.positionAt(stdSource.lastIndexOf('".txt"')));
  assert.equal(selectedHelp?.activeParameter,1);
  assert.equal(selectedHelp?.signatures[0].parameters?.[1].label,'suffix?: string');
  assert.ok(await document.save());
  assert.equal(await vscode.commands.executeCommand('tsrust.checkFile',entryUri),0);
  const stdEnd=new Promise<number | undefined>(resolve=>{
    const subscription=vscode.tasks.onDidEndTaskProcess(event=>{if(event.execution.task.source==='Tsrust'){subscription.dispose();resolve(event.exitCode);}});
  });
  assert.ok(await vscode.commands.executeCommand('tsrust.runFile',entryUri));
  assert.equal(await Promise.race([stdEnd,delay(60000).then(()=>{throw new Error('Std run task timed out');})]),0);
  console.log('Tsrust std E2E: alias/stored documentation, imported factory callable hover and parameter help, CLI checking and native task execution passed.');
  console.log('Tsrust module E2E: imported alias hover, cross-file definitions/references, unsaved dependency diagnostic/fix, import formatting, CLI check and native run passed.');
}
