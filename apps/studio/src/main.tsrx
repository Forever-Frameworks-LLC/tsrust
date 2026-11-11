import { View, Text, Button, Dock, Resizable, ResizablePanel, Node, signal, computed, run, setTheme, registerThemes, useTheme } from 'std:ui';
import { currentDir, args } from 'std:env';
import { extname } from 'std:path';
import { OpenFile, openWorkspace, selectFile } from './workspace.tsr';
import { dirty } from './documents.tsr';
import { Recovery, recoveryPath, restoreRecovery } from './recovery.tsr';
import { ProjectTree } from './browser.tsrx';
import { FileSearch } from './navigation.tsrx';
import { ContentSearch } from './search-panel.tsrx';
import { Chrome, closeTab } from './chrome.tsrx';
import { FilePanel } from './editor.tsrx';
import { Panels } from './panels.tsrx';
import { configPath, loadConfig } from './settings.tsr';
import studioThemes from './themes/studio.json';
import communityThemes from './themes/community.json';

function App(path: string): Node {
  const initial = unwrap(openWorkspace(path));
  let journal: Recovery = {
    path: '', baseline: '', ready: false
  };
  match (recoveryPath(initial.root)) {
    Ok(path) => {
      match (restoreRecovery(initial, path)) {
        Ok(value) => {
          journal = value;
        },
        Err(error) => {
          initial.error = error;
        }
      }
    },
    Err(error) => {
      initial.error = error;
    }
  }

  const recovery = signal(journal);
  const workspace = signal(initial);
  const sidebar = signal(true);
  const mode = signal('files');
  const font = signal(14);
  const panel = signal('');
  match (configPath()) {
    Ok(path) => {
      match (loadConfig(path)) {
        Ok(config) => {
          font.set(config.fontSize);
          useTheme(config.theme);
        },
        Err(message) => {
          const next = workspace.get();
          next.error = clone(message);
          workspace.set(next);
        }
      }
    },
    Err(message) => {
      const next = workspace.get();
      next.error = clone(message);
      workspace.set(next);
    }
  }

  const browser = ProjectTree(workspace.get().root, workspace, font);
  const fileSearch = FileSearch(workspace.get().root, workspace, font);
  const contentSearch = ContentSearch(workspace);
  const bottomPanel = Panels(panel);
  const error = computed<string>(() => workspace.get().error);
  const status = computed<string>(() => {
    const state = workspace.get();
    for (const file of state.files) {
      if (file.document.path === state.active) return (file.kind === 'image' ? 'Image' : extname(file.document.path).slice(1) + ' • UTF-8') + (dirty(file.document) ? ' • Unsaved' : '');
    }

    return 'No file open';
  });
  const chrome = Chrome(workspace, sidebar, mode, font, panel, recovery);
  return <View className="flex-col size-full bg-background text-foreground">
    {chrome}
    <View className="flex-row grow min-h-0">
      <View className="flex-col items-center w-12 py-2 gap-2 shrink-0 border-r border-border bg-muted">
        <Button icon="files" variant="ghost" selected={sidebar.get() && mode.get() === 'files'} tooltip="Files" aria-label="Files" onPress={() => { sidebar.set(mode.get() !== 'files' || !sidebar.get()); mode.set('files'); }} />
        <Button icon="search" variant="ghost" selected={sidebar.get() && mode.get() === 'search'} tooltip="Search contents" aria-label="Search contents" onPress={() => { sidebar.set(mode.get() !== 'search' || !sidebar.get()); mode.set('search'); }} />
        <Button icon="git-branch" variant="ghost" tooltip="Source Control (planned)" aria-label="Source Control" onPress={() => { panel.set('git'); }} />
        <Button icon="bug" variant="ghost" tooltip="Run and Debug (planned)" aria-label="Run and Debug" onPress={() => { panel.set('debug'); }} />
        <Button icon="blocks" variant="ghost" tooltip="Extensions (planned)" aria-label="Extensions" onPress={() => { panel.set('extensions'); }} />
        <Button icon="bot" variant="ghost" tooltip="AI Chat (planned)" aria-label="AI Chat" onPress={() => { panel.set('chat'); }} />
      </View>
      <Resizable className="grow min-w-0 min-h-0">
        <ResizablePanel size={248} minSize={160} maxSize={480} visible={sidebar.get()}>
          <View className="flex-col size-full bg-muted border-r border-border">
            <View hidden={mode.get() !== 'files'} className="flex-col size-full min-h-0">{browser}</View>
            <View hidden={mode.get() !== 'filesearch'} className="flex-col size-full min-h-0">{fileSearch}</View>
            <View hidden={mode.get() !== 'search'} className="flex-col size-full min-h-0">{contentSearch}</View>
          </View>
        </ResizablePanel>
        <ResizablePanel minSize={280}>
          <View className="flex-col size-full min-w-0 min-h-0">
            <Resizable vertical={true} className="size-full min-w-0 min-h-0">
              <ResizablePanel minSize={160}>
                <View className="flex-col size-full min-w-0 min-h-0 bg-background">
                  <View hidden={workspace.get().files.length !== 0} className="flex-col grow items-center justify-center">
                    <Button icon="file-search" variant="ghost" onPress={() => { mode.set('filesearch'); sidebar.set(true); }}>Open a file</Button>
                  </View>
                  <View hidden={workspace.get().files.length === 0} className="flex-col grow min-h-0">
                    <Dock value={workspace.get().active} tabbed={true} className="size-full" onChange={(path) => { const next = workspace.get(); if (next.active !== path && selectFile(next, clone(path))) workspace.set(next); }} onClose={(path) => { closeTab(workspace, clone(path)); }}>
                      {...workspace.get().files.map((file: ReadOnly<OpenFile>) => FilePanel(file, workspace, font, recovery))}
                    </Dock>
                  </View>
                  <View hidden={workspace.get().error === ''} className="px-3 py-2"><Text className="text-destructive">{error.get()}</Text></View>
                </View>
              </ResizablePanel>
              <ResizablePanel size={220} minSize={100} maxSize={480} visible={panel.get() !== ''}>{bottomPanel}</ResizablePanel>
            </Resizable>
          </View>
        </ResizablePanel>
      </Resizable>
    </View>
    <View className="flex-row items-center gap-3 px-2 h-6 shrink-0 bg-muted border-t border-border">
      <Button icon="terminal" variant="ghost" size="xsmall" tooltip="Toggle terminal" aria-label="Toggle terminal" onPress={() => { panel.set(panel.get() === 'terminal' ? '' : 'terminal'); }} />
      <Button icon="circle-alert" variant="ghost" size="xsmall" tooltip="Problems: language service not connected" onPress={() => { panel.set('problems'); }} />
      <Text className="text-xs">LSP: not connected</Text><View className="grow" /><Text className="text-xs">{status.get()}</Text>
    </View>
  </View>;
}

function main() {
  setTheme('dark');
  registerThemes(studioThemes);
  registerThemes(communityThemes);
  useTheme('Studio Dark');
  const parameters = unwrap(args());
  const path = parameters.length > 1 ? clone(parameters[1]) : unwrap(currentDir());
  unwrap(run(App(path), 'Tsrust Studio', {
    titlebar: 'custom', width: 1120, height: 780, minWidth: 720, minHeight: 520
  }));
}
