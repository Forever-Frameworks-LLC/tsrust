import * as assert from 'node:assert/strict';
import * as vscode from 'vscode';

export async function testAgreement(folder: vscode.Uri): Promise<void> {
  const uri = vscode.Uri.joinPath(folder, 'agreement.tsr');
  const head = 'import { ValidationError } from "std:schema";\nfunction inspect(parsed:Result<string,ValidationError>){';
  const schemaHead = 'import { s } from "std:schema";\nfunction main(){const schema=s.string();const parsed=schema.safeParse("hello");';
  await vscode.workspace.fs.writeFile(uri, Buffer.from(`${head}}`));
  const document = await vscode.workspace.openTextDocument(uri);
  const editor = await vscode.window.showTextDocument(document, { preview: false });
  const cases: [string, string[], string?][] = [
    ['parsed.', ['payload', 'match']],
    ['console.log(parsed.pa', ['payload']],
    ['match(parsed){Ok(value)=>{}Err(error)=>{error.', ['issues', 'truncated']],
    ['match(parsed){Ok(value)=>{}Err(error)=>{error.issues[0].', ['code', 'message', 'path']],
    ['parsed.', ['success'], schemaHead],
    ['if(parsed.success){parsed.', ['data'], schemaHead],
    ['if(!parsed.success){parsed.error.', ['issues', 'truncated'], schemaHead],
    ['if(!parsed.success){parsed.error.issues[0].', ['code', 'message', 'path'], schemaHead],
  ];
  for (const [typed, required, sourceHead = head] of cases) {
    assert.ok(await editor.edit(edit => edit.replace(new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), `${sourceHead}}`)));
    let offset = document.getText().lastIndexOf('}');
    for (const character of typed) {
      assert.ok(await editor.edit(edit => edit.insert(document.positionAt(offset), character)));
      offset += character.length;
    }
    // No settling delay or retry: the first answer after typing must be useful.
    const items = await vscode.commands.executeCommand<vscode.CompletionList>('vscode.executeCompletionItemProvider', uri, document.positionAt(offset));
    const labels = items?.items.map(item => typeof item.label === 'string' ? item.label : item.label.label) ?? [];
    for (const name of required) assert.ok(labels.includes(name), `${typed}: missing ${name}; got ${labels.join(', ')}`);
    if (sourceHead === schemaHead) for (const name of ['payload', 'match']) assert.ok(!labels.includes(name), `${typed}: validation union incorrectly offers ${name}`);
    for (const name of ['console', 'schema', 'parsed', 'main']) assert.ok(!labels.includes(name), `${typed}: unrelated ${name}`);
  }
  const valid = `${head}console.log(parsed.payload);}`;
  assert.ok(await editor.edit(edit => edit.replace(new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), valid)));
  const hovers = await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider', uri, document.positionAt(document.getText().indexOf('payload') + 1));
  const hover = hovers?.flatMap(item => item.contents.map(content => typeof content === 'string' ? content : content.value)).join('\n') ?? '';
  assert.ok(hover.includes('payload:') && hover.includes('string') && hover.includes('ValidationError'), hover);
  console.log('Tsrust agreement E2E: immediate Result/prefix/error/indexed completions, payload hover and separate validation-union narrowing passed.');
}
