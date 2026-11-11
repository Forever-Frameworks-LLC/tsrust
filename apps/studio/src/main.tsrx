import { accessibleName, message, initializeTranslations, localizeError } from './localization.tsr';
import { View, Text, Button, Dock, Resizable, ResizablePanel, Node, signal, computed, run, setTheme, registerThemes, useTheme, setLocale, translate, t } from 'std:ui';
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
import { Region } from './layout.tsrx';
import { Panels, ChatPanel } from './panels.tsrx';
import { configPath, loadConfig, defaultConfig } from './settings.tsr';
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
  const chat = signal(false);
  const settings = signal(defaultConfig());
  match (configPath()) {
    Ok(path) => {
      match (loadConfig(path)) {
        Ok(config) => {
          setLocale(config.locale);
          font.set(config.fontSize);
          useTheme(clone(config.theme));
          settings.set(config);
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
  const panelView = Panels(panel);
  const chatView = ChatPanel(chat);
  const error = computed<string>(() => localizeError(workspace.get().error));
  const status = computed<string>(() => {
    const state = workspace.get();
    for (const file of state.files) {
      if (file.document.path === state.active) return (file.kind === 'image' ? translate('Image') : extname(file.document.path).slice(1) + ' • UTF-8') + (dirty(file.document) ? ' • ' + translate('Unsaved') : '');
    }

    return translate('No file open');
  });
  const chrome = Chrome(workspace, sidebar, mode, font, panel, recovery, settings);
  const editor = <View className="flex-col size-full min-w-0 min-h-0 bg-background">
                  <View hidden={workspace.get().files.length !== 0} className="flex-col grow items-center justify-center">
                    <Button icon="file-search" variant="ghost" onPress={() => { mode.set('filesearch'); sidebar.set(true); }}>{t("Open a file")}</Button>
                  </View>
                  <View hidden={workspace.get().files.length === 0} className="flex-col grow min-h-0">
                    <Dock value={workspace.get().active} tabbed={true} className="size-full" onChange={(path) => { const next = workspace.get(); if (next.active !== path && selectFile(next, clone(path))) workspace.set(next); }} onClose={(path) => { closeTab(workspace, clone(path)); }}>
                      {...workspace.get().files.map((file: ReadOnly<OpenFile>) => FilePanel(file, workspace, font, recovery))}
                    </Dock>
                  </View>
                  <View hidden={workspace.get().error === ''} className="px-3 py-2"><Text className="text-destructive">{error.get()}</Text></View>
                </View>;
  const panelPosition = computed<string>(() => settings.get().panelPosition);
  const panelVisible = computed<boolean>(() => panel.get() !== '');
  const chatPosition = computed<string>(() => settings.get().chatPosition);
  const chatVisible = computed<boolean>(() => chat.get());
  const navigationPosition = computed<string>(() => settings.get().navigationSide);
  const navigationVisible = computed<boolean>(() => sidebar.get());
  const panelArea = Region(editor, panelView, panelPosition, panelVisible, 220, 100);
  const chatArea = Region(panelArea, chatView, chatPosition, chatVisible, 320, 160);
  const explorer = <View className="flex-col size-full min-h-0">
    <View hidden={mode.get() !== 'files'} className="flex-col size-full min-h-0">{browser}</View>
    <View hidden={mode.get() !== 'filesearch'} className="flex-col size-full min-h-0">{fileSearch}</View>
    <View hidden={mode.get() !== 'search'} className="flex-col size-full min-h-0">{contentSearch}</View>
  </View>;
  const navigationArea = Region(chatArea, explorer, navigationPosition, navigationVisible, 248, 160);
  const workbench = <View id="workbench" className="size-full min-w-0 min-h-0">
    {navigationArea}
  </View>;
  const rail = <View id="activity-bar" className={settings.get().activityBarSide === 'left' ? 'flex-col items-center w-12 h-full py-2 gap-2 shrink-0 border-r border-border bg-muted' : 'flex-col items-center w-12 h-full py-2 gap-2 shrink-0 border-l border-border bg-muted'}>
        <Button icon="files" variant="ghost" selected={sidebar.get() && mode.get() === 'files'} tooltip={message("Files")} aria-label={accessibleName("Files")} onPress={() => { sidebar.set(mode.get() !== 'files' || !sidebar.get()); mode.set('files'); }} />
        <Button icon="search" variant="ghost" selected={sidebar.get() && mode.get() === 'search'} tooltip={message("Search contents")} aria-label={accessibleName("Search contents")} onPress={() => { sidebar.set(mode.get() !== 'search' || !sidebar.get()); mode.set('search'); }} />
        <Button icon="git-branch" variant="ghost" selected={panel.get() === 'git'} tooltip={message("Source Control (planned)")} aria-label={accessibleName("Source Control")} onPress={() => { panel.set(panel.get() === 'git' ? '' : 'git'); }} />
        <Button icon="bug" variant="ghost" selected={panel.get() === 'debug'} tooltip={message("Run and Debug (planned)")} aria-label={accessibleName("Run and Debug")} onPress={() => { panel.set(panel.get() === 'debug' ? '' : 'debug'); }} />
        <Button icon="blocks" variant="ghost" selected={panel.get() === 'extensions'} tooltip={message("Extensions (planned)")} aria-label={accessibleName("Extensions")} onPress={() => { panel.set(panel.get() === 'extensions' ? '' : 'extensions'); }} />
        <Button icon="bot" variant="ghost" selected={chat.get()} tooltip={message("AI Chat (planned)")} aria-label={accessibleName("AI Chat")} onPress={() => { chat.set(!chat.get()); }} />
      </View>;
  return <View className="flex-col size-full bg-background text-foreground">
    {chrome}
    <View className="grow min-h-0">
      <Resizable reverse={settings.get().activityBarSide === 'right'} className="size-full min-w-0 min-h-0">
        <ResizablePanel size={48} minSize={48} maxSize={48} fixed={true}>{rail}</ResizablePanel>
        <ResizablePanel minSize={280}>{workbench}</ResizablePanel>
      </Resizable>
    </View>
    <View className="flex-row items-center gap-3 px-2 h-6 shrink-0 bg-muted border-t border-border">
      <Button icon="terminal" variant="ghost" size="xsmall" tooltip={message("Toggle terminal")} aria-label={accessibleName("Toggle terminal")} onPress={() => { panel.set(panel.get() === 'terminal' ? '' : 'terminal'); }} />
      <Button icon="circle-alert" variant="ghost" size="xsmall" tooltip={message("Problems: language service not connected")} onPress={() => { panel.set('problems'); }} />
      <Text className="text-xs">{t("LSP: not connected")}</Text><View className="grow" /><Text className="text-xs">{status.get()}</Text>
    </View>
  </View>;
}

function main() {
  initializeTranslations();
  setLocale('en');
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
