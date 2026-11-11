import { View, Text, Button, Tabs, Resizable, ResizablePanel, Node, Signal, signal, computed, run, setTheme, registerThemes, useTheme } from 'std:ui';
import { currentDir } from 'std:env';
import { join } from 'std:path';
import { Workspace, OpenFile, openWorkspace, openFile, selectFile } from './workspace.tsr';
import { ProjectTree } from './browser.tsrx';
import { FileSearch } from './navigation.tsrx';
import { Chrome } from './chrome.tsrx';
import { FileTab, FilePanel } from './editor.tsrx';
import studioThemes from './themes/studio.json';

function App(): Node {
  const initial = unwrap(openWorkspace(unwrap(currentDir())));
  const initialFile = join(initial.root, 'src', 'main.tsrx');
  match (openFile(initial, initialFile)) {
    Ok(_) => {},
    Err(message) => {
      initial.error = clone(message);
    }
  }

  const workspace = signal(initial);
  const sidebar = signal(true);
  const mode = signal('files');
  const font = signal(14);
  const browser = ProjectTree(initial.root, workspace, font);
  const search = FileSearch(initial.root, workspace, font);
  const error = computed<string>(() => workspace.get().error);
  const chrome = Chrome(workspace, sidebar, mode, font);
  return <View className="flex-col size-full bg-background text-foreground">
    {chrome}
    <View className="flex-row grow min-h-0">
      <View className="flex-col items-center w-12 py-2 gap-2 shrink-0 border-r border-border bg-muted">
        <Button icon="files" variant="ghost" selected={sidebar.get() && mode.get() === 'files'} tooltip="Files" aria-label="Files" onPress={() => { sidebar.set(mode.get() !== 'files' || !sidebar.get()); mode.set('files'); }} />
        <Button icon="search" variant="ghost" selected={sidebar.get() && mode.get() === 'search'} tooltip="Find file" aria-label="Find file" onPress={() => { sidebar.set(mode.get() !== 'search' || !sidebar.get()); mode.set('search'); }} />
      </View>
      <Resizable className="grow min-w-0 min-h-0">
        <ResizablePanel size={248} minSize={160} maxSize={480} visible={sidebar.get()}>
          <View className="flex-col size-full bg-muted border-r border-border">
            <View hidden={mode.get() !== 'files'} className="flex-col size-full min-h-0">{browser}</View>
            <View hidden={mode.get() !== 'search'} className="flex-col size-full min-h-0">{search}</View>
          </View>
        </ResizablePanel>
        <ResizablePanel minSize={280}>
          <View className="flex-col size-full min-w-0 min-h-0 bg-background">
            <Tabs value={workspace.get().active} onChange={(path) => { const next = workspace.get(); if (selectFile(next, path)) workspace.set(next); }} size="small" variant="tab" menu={true} className="shrink-0">
              {...workspace.get().files.map((file: ReadOnly<OpenFile>) => FileTab(file, workspace))}
            </Tabs>
            <View hidden={workspace.get().files.length !== 0} className="flex-col grow items-center justify-center">
              <Button icon="search" variant="ghost" onPress={() => { mode.set('search'); sidebar.set(true); }}>Open a file</Button>
            </View>
            <View className="flex-col grow min-h-0">
              {...workspace.get().files.map((file: ReadOnly<OpenFile>) => FilePanel(file, workspace, font))}
            </View>
            <View hidden={workspace.get().error === ''} className="px-3 py-2"><Text className="text-destructive">{error.get()}</Text></View>
          </View>
        </ResizablePanel>
      </Resizable>
    </View>
  </View>;
}

function main() {
  setTheme('dark');
  registerThemes(studioThemes);
  useTheme('Studio Dark');
  unwrap(run(App(), 'Tsrust Studio', {
    titlebar: 'custom', width: 1120, height: 780, minWidth: 720, minHeight: 520
  }));
}
