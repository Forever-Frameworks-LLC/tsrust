import { View, Button, CodeEditor, Tab, TabSuffix, Node, Signal, computed } from 'std:ui';
import { extname } from 'std:path';
import { Workspace, OpenFile } from './workspace.tsr';
import { closeTab } from './chrome.tsrx';

export function FileTab(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>): Node {
  const path = clone(file.document.path);
  return <Tab value={clone(path)} label={clone(file.name)} accessibilityLabel={clone(path)}>
    <TabSuffix><Button icon="x" variant="ghost" size="xsmall" aria-label={'Close ' + file.name} tooltip={'Close ' + file.name} onPress={() => { closeTab(workspace, path); }} /></TabSuffix>
  </Tab>;
}

export function FilePanel(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>, font: Signal<int>): Node {
  const path = clone(file.document.path);
  const editorClass = computed<string>(() => 'grow min-h-0 text-[' + font.get().toString() + 'px]');
  return <View id={clone(file.document.session)} hidden={workspace.get().active !== path} className="flex-col size-full min-h-0">
    <CodeEditor id={clone(file.document.path)} language={extname(file.document.path).slice(1)} value={clone(file.document.text)} readOnly={true} appearance={false} bordered={false} accessibilityLabel={clone(file.document.path)} className={editorClass.get()} />
  </View>;
}
