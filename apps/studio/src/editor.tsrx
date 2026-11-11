import { View, Button, CodeEditor, Markdown, Image, Tab, TabSuffix, Node, Signal, computed, signal } from 'std:ui';
import { extname } from 'std:path';
import { Workspace, OpenFile } from './workspace.tsr';
import { closeTab } from './chrome.tsrx';
import { dirty } from './documents.tsr';
import { Recovery } from './recovery.tsr';
import { updateDraft } from './document-actions.tsr';

export function FileTab(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>): Node {
  const path = clone(file.document.path);
  const name = clone(file.name);
  const labelPath = clone(path);
  const label = computed<string>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === labelPath);
    return name + (current !== undefined && dirty(current.document) ? ' •' : '');
  });
  return <Tab value={clone(path)} label={label.get()} accessibilityLabel={clone(path)}>
    <TabSuffix><Button icon="x" variant="ghost" size="xsmall" aria-label={'Close ' + file.name} tooltip={'Close ' + file.name} onPress={() => { closeTab(workspace, path); }} /></TabSuffix>
  </Tab>;
}

export function FilePanel(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>, font: Signal<int>, recovery: Signal<Recovery>): Node {
  const path = clone(file.document.path);
  const editPath = clone(path);
  const textPath = clone(path);
  const text = computed<string>(() => {
    const current = workspace.get().files.find((entry) => entry.document.path === textPath);
    return current === undefined ? '' : clone(current.document.text);
  });
  const preview = signal(false);
  const markdown = extname(path).toLowerCase() === '.md';
  const editorClass = computed<string>(() => 'grow min-h-0 text-[' + font.get().toString() + 'px]');
  if (file.kind === 'image') {
    return <View id={clone(file.document.session)} hidden={workspace.get().active !== path} className="flex-col size-full min-h-0 p-4">
      <Image src={clone(file.document.path)} fit="contain" alt={'Unable to decode ' + file.name} className="size-full" />
    </View>;
  }

  return <View id={clone(file.document.session)} hidden={workspace.get().active !== path} className="flex-col size-full min-h-0">
    <View hidden={!markdown} className="flex-row justify-end shrink-0 border-b border-border px-2">
      <Button icon="code" variant="ghost" size="xsmall" selected={!preview.get()} tooltip="Markdown source" aria-label="Markdown source" onPress={() => { preview.set(false); }} />
      <Button icon="eye" variant="ghost" size="xsmall" selected={preview.get()} tooltip="Markdown preview" aria-label="Markdown preview" onPress={() => { preview.set(true); }} />
    </View>
    <View hidden={preview.get()} className="flex-col grow min-h-0">
      <CodeEditor id={clone(file.document.path)} language={extname(file.document.path).slice(1)} value={text.get()} onChange={(value) => { updateDraft(workspace, recovery, editPath, value); }} appearance={false} bordered={false} accessibilityLabel={clone(file.document.path)} className={editorClass.get()} />
    </View>
    <View hidden={!preview.get()} className="grow min-h-0 overflow-y-auto p-4"><Markdown content={text.get()} scrollable={true} /></View>
  </View>;
}
