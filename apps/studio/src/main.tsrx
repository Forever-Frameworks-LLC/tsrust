import { accessibleName, message, initializeTranslations } from './lib/localization.tsr';
import { View, Text, Button, Resizable, ResizablePanel, Node, signal, computed, run, setTheme, registerThemes, useTheme, setLocale, translate, t } from 'std:ui';
import { currentDir, args, get } from 'std:env';
import { Channel, Sender } from 'std:futures';
import { CheckResult, projectCheckWorker } from './services/project-check.tsr';
import { CheckPanel } from './components/tasks/check-panel.tsrx';
import { extname } from 'std:path';
import { openWorkspace } from './lib/workspace/workspace.tsr';
import { dirty } from './lib/workspace/documents.tsr';
import { Recovery, recoveryPath, restoreRecovery } from './lib/workspace/recovery.tsr';
import { ProjectTree } from './components/explorer/browser.tsrx';
import { FileSearch } from './components/explorer/navigation.tsrx';
import { ContentSearch } from './components/explorer/search-panel.tsrx';
import { Chrome } from './components/workbench/chrome.tsrx';
import { EditorWorkspace } from './components/workbench/workspace-view.tsrx';
import { zoomFontSize } from './lib/workspace/zoom.tsr';
import { Region } from './components/workbench/layout.tsrx';
import { Panels, ChatPanel } from './components/workbench/panels.tsrx';
import { configPath, loadConfig, defaultConfig } from './lib/settings/settings.tsr';
import studioThemes from './assets/themes/studio.json';
import communityThemes from './assets/themes/community.json';

function App(path: string, requests: Sender<string>, shortcutRequests: Sender<string>, cancellations: Sender<string>, results: Channel<CheckResult>): Node {
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
          font.set(zoomFontSize(config.fontSize, workspace.get().zoomLevel));
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

  const browser = ProjectTree(workspace.get().root, workspace, font, recovery, settings);
  const fileSearch = FileSearch(workspace.get().root, workspace, font, recovery);
  const contentSearch = ContentSearch(workspace, recovery);
  const panelView = Panels(panel, CheckPanel(panel, requests, shortcutRequests, cancellations, results, font));
  const chatView = ChatPanel(chat);
  const status = computed<string>(() => {
    const state = workspace.get();
    for (const file of state.files) {
      if (file.document.path === state.active) return (file.kind === 'image' ? translate('Image') : extname(file.document.path).slice(1) + ' • UTF-8') + (dirty(file.document) ? ' • ' + translate('Unsaved') : '');
    }

    return translate('No file open');
  });
  const chrome = Chrome(workspace, sidebar, mode, font, panel, recovery, settings);
  const editor = EditorWorkspace(workspace, font, recovery, mode, sidebar);
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

async function main(): Task<void> {
  initializeTranslations();
  setLocale('en');
  setTheme('dark');
  registerThemes(studioThemes);
  registerThemes(communityThemes);
  useTheme('Studio Dark');
  const parameters = unwrap(args());
  const path = parameters.length > 1 ? clone(parameters[1]) : unwrap(currentDir());
  const requests = new Channel<string>(1);
  const results = new Channel<CheckResult>(1);
  const actions = requests.sender();
  const shortcuts = requests.sender();
  const cancellations = requests.sender();
  const replies = results.sender();
  const configured = unwrap(get('TSRUST'));
  const program: string = configured ?? 'tsrust';
  const worker = projectCheckWorker(clone(path), program, requests, replies).start();
  try {
    unwrap(run(App(path, actions, shortcuts, cancellations, results), 'Tsrust Studio', {
      titlebar: 'custom', width: 1120, height: 780, minWidth: 720, minHeight: 520
    }));
  } finally {
    worker.cancel();
    try {
      await worker;
    } catch (_) {}
  }
}
