import { View, Text, CodeEditor, Tabs, Tab, StatusBar, StatusBarLeft, Node, Signal, signal, computed, run, setTheme } from 'std:ui';
import { currentDir } from 'std:env';
import { join } from 'std:path';
import { Workspace, OpenFile, openWorkspace, openFile } from './workspace.tsr';
import { ProjectTree } from './browser.tsrx';

function FilePanel(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>): Node {
  const path = clone(file.path);
  return <View hidden={workspace.get().active !== path} className="flex-col grow min-h-0">
    <CodeEditor id={clone(file.path)} language={clone(file.language)} value={clone(file.text)} readOnly={true} appearance={false} bordered={false} accessibilityLabel={clone(file.path)} className="grow min-h-0" />
  </View>;
}

function App(): Node {
  const initial = unwrap(openWorkspace(unwrap(currentDir())));
  const initialFile = join(initial.root, 'src', 'main.tsrx');
  match (openFile(initial, initialFile)) {
    Ok(_) => {},
    Err(message) => {
      initial.message = clone(message);
    }
  }

  const workspace = signal(initial);
  const status = computed<string>(() => workspace.get().message);
  const browser = ProjectTree(initial.root, workspace);
  return <View className="flex-col size-full">
    <View className="flex-row grow min-h-0">
      <View className="flex-col w-64 shrink-0 border-r min-h-0">
        {browser}
      </View>
      <View className="flex-col grow min-w-0 min-h-0">
      <Tabs value={workspace.get().active} onChange={(path) => {
        const next = workspace.get();
        next.active = clone(path);
        next.message = 'Read-only preview • ' + path;
        workspace.set(next);
      }} size="small" variant="tab" className="shrink-0">
        {...workspace.get().files.map((file: ReadOnly<OpenFile>) => <Tab value={clone(file.path)} label={clone(file.name)} accessibilityLabel={clone(file.path)} />)}
      </Tabs>
      <View className="flex-col grow min-h-0">
        {...workspace.get().files.map((file: ReadOnly<OpenFile>) => FilePanel(file, workspace))}
      </View>
      </View>
    </View>
    <StatusBar><StatusBarLeft><Text>{status.get()}</Text></StatusBarLeft></StatusBar>
  </View>;
}

function main() {
  setTheme('dark');
  unwrap(run(App(), 'Tsrust Studio', {
    width: 1120, height: 780, minWidth: 720, minHeight: 520
  }));
}
