import { currentDir, args, get } from 'std:env';
import { canonicalize } from 'std:fs';
import { Channel } from 'std:futures';
import { extname } from 'std:path';
import {
  View,
  Text,
  Button,
  Resizable,
  ResizablePanel,
  Node,
  signal,
  computed,
  useId,
  run,
  setTheme,
  registerThemes,
  useTheme,
  setLocale,
  translate,
  t
} from 'std:ui';

import { SaveConflicts } from './components/editor/save-conflict.tsrx';
import { ProjectTree } from './components/explorer/browser.tsrx';
import { FileSearch } from './components/explorer/navigation.tsrx';
import { ContentSearch } from './components/explorer/search-panel.tsrx';
import { CheckPanel } from './components/tasks/check-panel.tsrx';
import { ProblemsPanel, CopyProblems } from './components/tasks/problems-panel.tsrx';
import { TerminalPanel, NewTerminal } from './components/tasks/terminal-panel.tsrx';
import { Chrome } from './components/workbench/chrome.tsrx';
import { LanguageStatus } from './components/workbench/language-status.tsrx';
import { Region } from './components/workbench/layout.tsrx';
import { Panels, ChatPanel } from './components/workbench/panels.tsrx';
import { UnsavedFiles } from './components/workbench/unsaved-files.tsrx';
import { WorkflowControls } from './components/workbench/workflow-controls.tsrx';
import { EditorWorkspace } from './components/workbench/workspace-view.tsrx';
import { accessibleName, message, initializeTranslations } from './lib/localization.tsr';
import { configPath, loadConfig, defaultConfig } from './lib/settings/settings.tsr';
import { dirty } from './lib/workspace/documents.tsr';
import { QueuedFormatting, FormattingRequest, FormattingResult } from './lib/workspace/formatting-actions.tsr';
import { Problem, collectProblems } from './lib/workspace/problems.tsr';
import { Recovery, recoveryPath, restoreRecovery } from './lib/workspace/recovery.tsr';
import { emptyTerminals, addTerminal } from './lib/workspace/terminals.tsr';
import { openWorkspace } from './lib/workspace/workspace.tsr';
import { zoomFontSize } from './lib/workspace/zoom.tsr';
import { LanguageMessage, LanguageUpdate } from './services/lsp/session.tsr';
import { CheckResult } from './services/project-check.tsr';
import { studioWorkers } from './services/workers.tsr';

import { LanguageState } from './services/lsp/controller.tsr';

import communityThemes from './assets/themes/community.json';
import studioThemes from './assets/themes/studio.json';

function App(
  path: string,
  requests: ReadOnly<Channel<string>>,
  results: Channel<CheckResult>,
  formattingRequests: ReadOnly<Channel<FormattingRequest>>,
  formattingResults: Channel<FormattingResult>,
  program: string,
  languageMessages: ReadOnly<Channel<LanguageMessage>>,
  languageUpdates: Channel<LanguageUpdate>
): Node {
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
  const checking = signal(false);
  const checkOutput = signal('');
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
  const contentSearch = ContentSearch(workspace, recovery, font, settings);
  const initialTerminals = emptyTerminals();
  unwrap(addTerminal(initialTerminals, useId()));
  const terminals = signal(initialTerminals);
  const terminalError = signal('');
  const problems = signal<Problem[]>([]);
  const problemCount = computed<int>(() => collectProblems(workspace.get(), problems.get(), terminalError.get()).filter((problem) => problem.severity === 'error').length);
  const warningCount = computed<int>(() => collectProblems(workspace.get(), problems.get(), terminalError.get()).filter((problem) => problem.severity === 'warning').length);
  const problemLabel = computed<string>(() => translate('Errors') + ': ' + problemCount.get().toString());
  const panelView = Panels(
    panel,
    CheckPanel(
      requests.sender(),
      results,
      font,
      checking,
      checkOutput,
      problems,
      panel
    ),
    TerminalPanel(
      workspace.get().root,
      font,
      terminals,
      terminalError,
      workspace,
      recovery
    ),
    NewTerminal(terminals, terminalError),
    ProblemsPanel(
      workspace,
      recovery,
      terminals,
      panel,
      problems,
      terminalError,
      font
    ),
    CopyProblems(workspace, problems, terminalError),
    font
  );
  const formatBusy = signal(false);
  const formatFocus = signal<QueuedFormatting[]>([]);
  const workflows = WorkflowControls(
    workspace,
    recovery,
    settings,
    panel,
    formattingRequests,
    formattingResults,
    requests,
    checking,
    checkOutput,
    problems,
    terminals,
    terminalError,
    program,
    formatBusy,
    formatFocus
  );
  const chatView = ChatPanel(chat);
  const statusClass = computed<string>(() => 'flex-row items-center gap-3 px-2 h-6 shrink-0 bg-muted border-t border-border text-[' + font.get().toString() + 'px]');
  const status = computed<string>(() => {
    const state = workspace.get();

    for (const file of state.files) {
      if (file.document.path === state.active) return (file.kind === 'image' ? translate('Image') : extname(file.document.path).slice(1) + ' • UTF-8') + (dirty(file.document) ? ' • ' + translate('Unsaved') : '');
    }

    return translate('No file open');
  });
  const chrome = Chrome(
    workspace,
    sidebar,
    mode,
    font,
    panel,
    recovery,
    settings
  );
  const languageState = signal<LanguageState>({
    generation: 0, ready: false, status: 'LSP: starting', detail: '', documents: [], version: 0, queries: [], readResults: []
  });
  const editor = EditorWorkspace(
    workspace,
    font,
    recovery,
    mode,
    sidebar,
    formatFocus,
    languageState,
    languageMessages.sender()
  );
  const panelPosition = computed<string>(() => settings.get().panelPosition);
  const panelVisible = computed<boolean>(() => panel.get() !== '');
  const chatPosition = computed<string>(() => settings.get().chatPosition);
  const chatVisible = computed<boolean>(() => chat.get());
  const navigationPosition = computed<string>(() => settings.get().navigationSide);
  const navigationVisible = computed<boolean>(() => sidebar.get());
  const panelArea = Region(
    editor,
    panelView,
    panelPosition,
    panelVisible,
    220,
    100
  );
  const chatArea = Region(
    panelArea,
    chatView,
    chatPosition,
    chatVisible,
    320,
    160
  );
  const explorer = <View className="flex-col size-full min-h-0">
    <View hidden={mode.get() !== 'files'} className="flex-col size-full min-h-0">
      {browser}
    </View>
    <View hidden={mode.get() !== 'filesearch'} className="flex-col size-full min-h-0">
      {fileSearch}
    </View>
    <View hidden={mode.get() !== 'search'} className="flex-col size-full min-h-0">
      {contentSearch}
    </View>
  </View>;
  const navigationArea = Region(
    chatArea,
    explorer,
    navigationPosition,
    navigationVisible,
    248,
    160
  );
  const workbench = <View id="workbench" className="size-full min-w-0 min-h-0">
    {navigationArea}
  </View>;
  const rail = <View id="activity-bar" className="flex-col items-center w-12 h-full py-2 gap-2 shrink-0 bg-muted">
    <Button
      icon="files"
      variant="ghost"
      selected={sidebar.get() && mode.get() === 'files'}
      tooltip={message("Files")}
      aria-label={accessibleName("Files")}
      onPress={() => {
        sidebar.set(mode.get() !== 'files' || !sidebar.get());
        mode.set('files');
      }}
    />
    <Button
      icon="search"
      variant="ghost"
      selected={sidebar.get() && mode.get() === 'search'}
      tooltip={message("Search contents")}
      aria-label={accessibleName("Search contents")}
      onPress={() => {
        sidebar.set(mode.get() !== 'search' || !sidebar.get());
        mode.set('search');
      }}
    />
    <Button
      icon="git-branch"
      variant="ghost"
      selected={panel.get() === 'git'}
      tooltip={message("Source Control (planned)")}
      aria-label={accessibleName("Source Control")}
      onPress={() => {
        panel.set(panel.get() === 'git' ? '' : 'git');
      }}
    />
    <Button
      icon="bug"
      variant="ghost"
      selected={panel.get() === 'debug'}
      tooltip={message("Run and Debug (planned)")}
      aria-label={accessibleName("Run and Debug")}
      onPress={() => {
        panel.set(panel.get() === 'debug' ? '' : 'debug');
      }}
    />
    <Button
      icon="blocks"
      variant="ghost"
      selected={panel.get() === 'extensions'}
      tooltip={message("Extensions (planned)")}
      aria-label={accessibleName("Extensions")}
      onPress={() => {
        panel.set(panel.get() === 'extensions' ? '' : 'extensions');
      }}
    />
    <Button
      icon="bot"
      variant="ghost"
      selected={chat.get()}
      tooltip={message("AI Chat (planned)")}
      aria-label={accessibleName("AI Chat")}
      onPress={() => {
        chat.set(!chat.get());
      }}
    />
  </View>;
  return (
    <View className="flex-col size-full bg-background text-foreground">
      {SaveConflicts(workspace, recovery, font)}
      {chrome}
      <View className="grow min-h-0">
        <Resizable reverse={settings.get().activityBarSide === 'right'} className="size-full min-w-0 min-h-0">
          <ResizablePanel size={48} minSize={48} maxSize={48} fixed={true}>
            {rail}
          </ResizablePanel>
          <ResizablePanel minSize={280}>
            {workbench}
          </ResizablePanel>
        </Resizable>
      </View>
      <View className={statusClass.get()}>
        <Button
          icon="circle-alert"
          variant="ghost"
          size="xsmall"
          tooltip={problemLabel.get()}
          aria-label={problemLabel.get()}
          className={'gap-1 text-[' + font.get().toString() + 'px]' + (problemCount.get() > 0 ? ' text-destructive' : '')}
          onPress={() => {
            panel.set('problems');
          }}
        >
          {problemCount.get().toString()}
        </Button>
        <Button
          icon="triangle-alert"
          variant="ghost"
          size="xsmall"
          tooltip={translate('Warnings') + ': ' + warningCount.get().toString()}
          aria-label={translate('Warnings') + ': ' + warningCount.get().toString()}
          className={'gap-1 text-[' + font.get().toString() + 'px]'}
          onPress={() => {
            panel.set('problems');
          }}
        >
          {warningCount.get().toString()}
        </Button>
        {LanguageStatus(
          workspace,
          problems,
          panel,
          font,
          languageMessages.sender(),
          languageMessages.sender(),
          languageUpdates,
          languageState
        )}
        {UnsavedFiles(workspace, recovery, font)}
        <Text>
          {status.get()}
        </Text>
        <View className="grow" />
        {workflows}
      </View>
    </View>
  );
}

async function main(): Task<void> {
  initializeTranslations();
  setLocale('en');
  setTheme('dark');
  registerThemes(studioThemes);
  registerThemes(communityThemes);
  useTheme('Studio Dark');
  const parameters = unwrap(args());
  const path = unwrap(canonicalize(parameters.length > 1 ? clone(parameters[1]) : unwrap(currentDir())));
  const requests = new Channel<string>(1);
  const results = new Channel<CheckResult>(1);
  const replies = results.sender();
  const formattingRequests = new Channel<FormattingRequest>(1);
  const formattingResults = new Channel<FormattingResult>(1);
  const formattingReplies = formattingResults.sender();
  const configured = unwrap(get('TSRUST'));
  const program: string = configured ?? 'tsrust';
  const languageMessages = new Channel<LanguageMessage>(8);
  const languageUpdates = new Channel<LanguageUpdate>(8);
  const languageReplies = languageUpdates.sender();
  const view = App(
    clone(path),
    requests,
    results,
    formattingRequests,
    formattingResults,
    clone(program),
    languageMessages,
    languageUpdates
  );
  const workers = studioWorkers(
    clone(path),
    program,
    requests,
    replies,
    formattingRequests,
    formattingReplies,
    languageMessages,
    languageReplies
  ).start();

  try {
    unwrap(run(view, 'Tsrust Studio', {
      titlebar: 'custom', width: 1120, height: 780, minWidth: 720, minHeight: 520
    }));
  } finally {
    workers.cancel();

    try {
      await workers;
    } catch (_) {}
  }
}
