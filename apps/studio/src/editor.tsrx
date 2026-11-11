import { View, Button, CodeEditor, Markdown, Image, DockPanel, Node, Signal, computed, signal } from 'std:ui';
import { extname } from 'std:path';
import { Workspace, OpenFile, selectFile } from './workspace.tsr';
import { dirty } from './documents.tsr';
import { Recovery } from './recovery.tsr';
import { updateDraft } from './document-actions.tsr';

export function FilePanel(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>, font: Signal<int>, recovery: Signal<Recovery>): Node {
  const path = clone(file.document.path);
  const editPath = clone(path);
  const focusPath = clone(path);
  const labelPath = clone(path);
  const name = clone(file.name);
  const label = computed<string>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === labelPath);
    return name + (current !== undefined && dirty(current.document) ? ' •' : '');
  });
  const closePath = clone(path);
  const closable = computed<boolean>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === closePath);
    return current === undefined || !dirty(current.document);
  });
  const textPath = clone(path);
  const text = computed<string>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === textPath);
    return current === undefined ? '' : clone(current.document.text);
  });
  const preview = signal(false);
  const markdown = extname(path).toLowerCase() === '.md';
  const editorClass = computed<string>(() => 'grow min-h-0 text-[' + font.get().toString() + 'px]');
  if (file.kind === 'image') {
    return <DockPanel value={clone(path)} title={label.get()} padded={false} splitControls={true} closable={closable.get()}><View id={clone(file.document.session)} className="flex-col size-full min-h-0 p-4">
      <Image src={clone(file.document.path)} fit="contain" alt={'Unable to decode ' + file.name} className="size-full" />
    </View></DockPanel>;
  }

  return <DockPanel value={clone(path)} title={label.get()} padded={false} splitControls={true} closable={closable.get()}><View id={clone(file.document.session)} className="flex-col size-full min-h-0">
    <View hidden={!markdown} className="flex-row justify-end shrink-0 border-b border-border px-2">
      <Button icon="code" variant="ghost" size="xsmall" selected={!preview.get()} tooltip="Markdown source" aria-label="Markdown source" onPress={() => { preview.set(false); }} />
      <Button icon="eye" variant="ghost" size="xsmall" selected={preview.get()} tooltip="Markdown preview" aria-label="Markdown preview" onPress={() => { preview.set(true); }} />
    </View>
    <View hidden={preview.get()} className="flex-col grow min-h-0">
      <CodeEditor id={clone(file.document.path)} language={extname(file.document.path).slice(1)} value={text.get()} onChange={(value) => { updateDraft(workspace, recovery, editPath, value); }} onFocus={() => { const next = workspace.get(); if (next.active !== focusPath && selectFile(next, clone(focusPath))) workspace.set(next); }} appearance={false} bordered={false} accessibilityLabel={clone(file.document.path)} className={editorClass.get()} />
    </View>
    <View hidden={!preview.get()} className="grow min-h-0 overflow-y-auto p-4"><Markdown content={text.get()} scrollable={true} /></View>
  </View></DockPanel>;
}
