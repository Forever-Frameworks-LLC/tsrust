import { DebugConsole, DebugSidebar, chooseDebugTarget, sendDebugControl, toggleCursorBreakpoint } from './components/tasks/debug-console.tsrx';
import { DebugCommand, DebugOutput } from './services/debug/session.tsr';
import { availableModelPaths, modelExtensions } from './services/ai/model-extensions.tsr';
import { toggleExtensions, ExtensionBrowser, ModelExtensions } from './components/workbench/model-extensions.tsrx';
import { restoreChats } from './services/ai/history.tsr';
import { currentDir, currentExe, args, get } from 'std:env';
import { canonicalize } from 'std:fs';
import { Channel, Sender } from 'std:futures';
import { extname, dirname, join } from 'std:path';
import {
  View,
  Ticker,
  Shortcut,
  Text,
  Button,
  Resizable,
  ResizablePanel,
  Node,
  signal,
  computed,
  controlInput,
  useId,
  run,
  setTheme,
  registerThemes,
  useTheme,
  setLocale,
  translate,
  t
} from 'std:ui';

import { ChatEditReview } from './components/ai/edit-review.tsrx';
import { SaveConflicts } from './components/editor/save-conflict.tsrx';
import { ProjectTree, RefreshExplorer } from './components/explorer/browser.tsrx';
import { DirectoryTree } from './lib/explorer/tree.tsr';
import { FileSearch } from './components/explorer/navigation.tsrx';
import { FileSearchState } from './lib/explorer/search-actions.tsr';
import { ContentSearch } from './components/explorer/search-panel.tsrx';
import { ReferencesPanel, ReferenceControls } from './components/tasks/references-panel.tsrx';
import { LanguageReferences } from './lib/language/references.tsr';
import { CheckPanel } from './components/tasks/check-panel.tsrx';
import { ProblemsPanel, CopyProblems } from './components/tasks/problems-panel.tsrx';
import { TerminalPanel, NewTerminal } from './components/tasks/terminal-panel.tsrx';
import { Chrome } from './components/workbench/chrome.tsrx';
import { LanguageStatus } from './components/workbench/language-status.tsrx';
import { Region } from './components/workbench/layout.tsrx';
import { Panels } from './components/workbench/panels.tsrx';
import { ChatPanel } from './components/ai/chat-panel.tsrx';
import { ModelRequest, ModelUpdate } from './lib/ai/protocol.tsr';
import { FileDeclarations } from './services/ai/context.tsr';
import { AiContext, initialAiState, syncInlinePreference, pollModel } from './services/ai/actions.tsr';
import { UnsavedFiles } from './components/workbench/unsaved-files.tsrx';
import { WorkflowControls } from './components/workbench/workflow-controls.tsrx';
import { EditorWorkspace } from './components/workbench/workspace-view.tsrx';
import { accessibleName, message, initializeTranslations } from './lib/localization.tsr';
import { StudioConfig, configPath, loadConfig, defaultConfig } from './lib/settings/settings.tsr';
import { dirty } from './lib/workspace/documents.tsr';
import { QueuedFormatting, FormattingRequest, FormattingResult } from './lib/workspace/formatting-actions.tsr';
import { Problem, collectProblems } from './lib/workspace/problems.tsr';
import { Recovery, recoveryPath, restoreRecovery } from './lib/workspace/recovery.tsr';
import { emptyTerminals, addTerminal } from './lib/workspace/terminals.tsr';
import { openWorkspace, SourceBreakpoint } from './lib/workspace/workspace.tsr';
import { zoomFontSize } from './lib/workspace/zoom.tsr';
import { LanguageMessage, LanguageUpdate } from './services/lsp/session.tsr';
import { CheckResult } from './services/project-check.tsr';
import { ContentSearchRequest, ContentSearchReply } from './services/content-search.tsr';
import { GitPanel, GitEditor } from './components/workbench/git-panel.tsrx';
import { DiffHunk, GitChange, GitRequest, GitReply } from './services/git.tsr';
import { studioWorkers } from './services/workers.tsr';

import { studioServers } from './lib/language/servers.tsr';
import { createLanguageState } from './services/lsp/controller.tsr';

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
  languageUpdates: Channel<LanguageUpdate>,
  modelRequests: ReadOnly<Channel<ModelRequest>>,
  modelUpdates: Channel<ModelUpdate>,
  searchRequests: ReadOnly<Channel<ContentSearchRequest>>,
  cancellations: ReadOnly<Channel<int>>,
  searchReplies: Channel<ContentSearchReply>,
  gitRequests: Sender<GitRequest>,
  gitReplies: Channel<GitReply>,
  downloads: Sender<string>,
  downloadReplies: Channel<string>,
  debugRequests: ReadOnly<Channel<DebugCommand>>,
  debugResults: Channel<DebugOutput>,
  initialConfig: StudioConfig,
  settingsError: string
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
  const filenameSearch = signal<FileSearchState>({
    root: clone(workspace.get().root), query: '', files: []
  });
  const font = signal(14);
  const panel = signal('');
  const references = signal<LanguageReferences>({
    root: clone(workspace.get().root), locations: []
  });
  const checking = signal(false);
  const checkOutput = signal('');
  const chat = signal(false);
  const restoredChats = restoreChats(workspace.get().root);
  const aiStore = signal(clone(restoredChats.store));
  const aiState = signal(initialAiState(clone(restoredChats.history), clone(restoredChats.error)));
  setLocale(initialConfig.locale);
  font.set(zoomFontSize(initialConfig.fontSize, workspace.get().zoomLevel));
  useTheme(clone(initialConfig.theme));
  const settings = signal(initialConfig);
  const modelPaths = signal<string[]>([]);

  if (settingsError !== '') {
    const next = workspace.get();
    next.error = settingsError;
    workspace.set(next);
  }

  const explorerRevision = signal(0);
  const refreshExplorer = RefreshExplorer(clone(workspace.get().root), workspace, recovery, explorerRevision);
  const projectTree = signal<DirectoryTree>({
    root: clone(workspace.get().root), directories: []
  });
  const browser = ProjectTree(
    workspace.get().root,
    workspace,
    font,
    recovery,
    settings,
    explorerRevision,
    projectTree.handle()
  );
  const contentSearch = ContentSearch(
    workspace,
    recovery,
    font,
    settings,
    searchRequests,
    cancellations.sender(),
    searchReplies
  );
  const initialTerminals = emptyTerminals();
  unwrap(addTerminal(initialTerminals, useId()));
  const terminals = signal(initialTerminals);
  const terminalError = signal('');
  const problems = signal<Problem[]>([]);
  const problemCount = computed<int>(() => collectProblems(workspace.get(), problems.get(), terminalError.get()).filter((problem) => problem.severity === 'error').length);
  const warningCount = computed<int>(() => collectProblems(workspace.get(), problems.get(), terminalError.get()).filter((problem) => problem.severity === 'warning').length);
  const problemLabel = computed<string>(() => translate('Errors') + ': ' + problemCount.get().toString());
  const languageState = signal(createLanguageState(studioServers(program, settings.get().languageServers)));
  const fileSearch = FileSearch(
    workspace,
    recovery,
    filenameSearch,
    font,
    languageState,
    languageMessages.sender()
  );
  const selectedExtension = signal(clone(modelExtensions()[0].name));
  const extensionBrowser = ExtensionBrowser(selectedExtension, font, modelPaths);
  const extensionDetail = ModelExtensions(
    settings,
    modelPaths,
    aiState,
    modelRequests.sender(),
    downloads,
    selectedExtension,
    font,
    downloadReplies
  );
  const debugOutput = signal('');
  const sourceBreakpoints = signal<SourceBreakpoint[]>(clone(workspace.get().sourceBreakpoints) ?? []);
  const debuggerRunning = signal(false);
  const debugTarget = signal('');
  const debuggerPaused = signal(false);
  const stepIntoControl = debugRequests.sender();
  const stepOutControl = debugRequests.sender();
  const debugControls = debugRequests.sender();
  const continueControl = debugRequests.sender();
  const stopControl = debugRequests.sender();
  const debugSidebar = DebugSidebar(
    debugRequests,
    debugResults,
    font,
    debugOutput,
    panel,
    {
      workspace: workspace.handle(), recovery: recovery.handle(), allowedRoot: clone(workspace.get().root), allowNavigation: true, focusEditor: true
    },
    sourceBreakpoints.handle(),
    debuggerRunning.handle(),
    debuggerPaused.handle(),
    debugTarget.handle()
  );
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
    ReferencesPanel(workspace, recovery, references, languageState, font),
    DebugConsole(debugOutput, font),
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
    formatFocus,
    font
  );
  const declarations = signal<FileDeclarations[]>([]);
  const aiContext: AiContext = {
    problems: problems.handle(), language: languageState.handle(), declarations: declarations.handle(), references: references.handle(), projectTree, state: aiState.handle(), settings: settings.handle(), commands: modelRequests.sender(), store: aiStore.handle(), workspace: workspace.handle(), allowRead: true, allowControl: true, allowInference: true
  };
  const chatView = ChatPanel(aiContext, font, aiState);
  const statusClass = computed<string>(() => 'flex-row items-center gap-3 px-2 h-6 shrink-0 bg-muted border-t border-border text-[' + font.get().toString() + 'px]');
  const status = computed<string>(() => {
    const state = workspace.get();

    for (const file of state.files) {
      if (file.document.path === state.active) return (file.kind === 'image' ? translate('Image') : file.kind === 'binary' ? translate('Binary') : file.kind === 'bytes' ? translate('Byte preview') : extname(file.document.path).slice(1) + ' • UTF-8') + (dirty(file.document) ? ' • ' + translate('Unsaved') : '');
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
    settings,
    filenameSearch,
    chat,
    aiState,
    aiStore
  );
  const editor = EditorWorkspace(
    workspace,
    font,
    recovery,
    formatFocus,
    languageState,
    languageMessages.sender(),
    aiState,
    modelRequests.sender(),
    references,
    panel,
    filenameSearch
  );
  const panelPosition = computed<string>(() => settings.get().panelPosition);
  const panelVisible = computed<boolean>(() => panel.get() !== '');
  const chatPosition = computed<string>(() => settings.get().chatPosition);
  const chatVisible = computed<boolean>(() => chat.get());
  const navigationPosition = computed<string>(() => settings.get().navigationSide);
  const navigationVisible = computed<boolean>(() => sidebar.get());
  const viewingDiff = signal(false);
  const gitSelection = signal<GitChange>({
    path: '', mode: '', status: ''
  });
  const gitPending = signal<GitRequest>({
    mode: 'status'
  });
  const gitOutput = signal('');
  const gitOurs = signal('');
  const gitTheirs = signal('');
  const gitBusy = signal(false);
  const gitError = signal('');
  const gitHunks = signal<DiffHunk[]>([]);
  const gitSidebar = GitPanel(
    workspace,
    recovery,
    font,
    mode,
    sidebar,
    viewingDiff,
    gitRequests,
    gitReplies,
    gitSelection.handle(),
    gitError.handle(),
    gitOutput.handle(),
    gitOurs.handle(),
    gitTheirs.handle(),
    gitBusy.handle(),
    gitPending.handle(),
    gitHunks.handle()
  );
  const gitEditor = GitEditor(
    workspace,
    recovery,
    font,
    gitSelection.handle(),
    gitPending.handle(),
    gitOutput.handle(),
    gitOurs.handle(),
    gitTheirs.handle(),
    gitBusy.handle(),
    gitError.handle(),
    gitHunks.handle()
  );
  const editingArea = <View className="size-full flex-col min-h-0">
    <View hidden={mode.get() === 'extensions' || (viewingDiff.get() && mode.get() === 'git')} className="size-full">
      {editor}
    </View>
    <View hidden={mode.get() !== 'extensions'} className="size-full min-h-0">
      {extensionDetail}
    </View>
    <View hidden={!viewingDiff.get() || mode.get() !== 'git'} className="size-full">
      {gitEditor}
    </View>
  </View>;
  const panelArea = Region(
    editingArea,
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
    <View hidden={mode.get() !== 'debug'} className="flex-col size-full min-h-0">
      {debugSidebar}
    </View>
    <View hidden={mode.get() !== 'extensions'} className="flex-col size-full min-h-0">
      {extensionBrowser}
    </View>
    <View hidden={mode.get() !== 'git'} className="flex-col size-full min-h-0">
      {gitSidebar}
    </View>
    <View hidden={mode.get() !== 'files'} className="flex-col size-full min-h-0">
      {browser}
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
  const referenceControls = ReferenceControls(
    workspace,
    recovery,
    references,
    languageState,
    panel,
    languageMessages.sender()
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

        if (sidebar.get()) controlInput('content-search', 'focus');
      }}
    />
    <Button
      icon="git-branch"
      variant="ghost"
      selected={sidebar.get() && mode.get() === 'git'}
      tooltip={message("Source Control")}
      aria-label={accessibleName("Source Control")}
      onPress={() => {
        sidebar.set(mode.get() !== 'git' || !sidebar.get());
        mode.set('git');
      }}
    />
    <Button
      icon="bug"
      variant="ghost"
      selected={sidebar.get() && mode.get() === 'debug'}
      tooltip={message("Run and Debug")}
      aria-label={accessibleName("Run and Debug")}
      onPress={() => {
        sidebar.set(mode.get() !== 'debug' || !sidebar.get());
        mode.set('debug');
      }}
    />
    <Button
      icon="blocks"
      variant="ghost"
      selected={sidebar.get() && mode.get() === 'extensions'}
      tooltipCommand="show-extensions"
      tooltip={message("Extensions")}
      aria-label={accessibleName("Extensions")}
      onPress={() => {
        toggleExtensions(sidebar, mode, settings, modelPaths);
      }}
    />
  </View>;
  return (
    <View className="flex-col size-full bg-background text-foreground">
      <Ticker
        interval={(aiState.get().phase === 'generating' || aiState.get().phase === 'inline') ? 50 : aiState.get().phase === 'ready' ? 1000 : 150}
        running={aiState.get().phase !== 'stopped' || aiState.get().queuedInline !== undefined || (aiState.get().inlineDisabled ?? false) === settings.get().inlineCompletions || (settings.get().inlineCompletions && settings.get().aiModelPath !== '' && aiState.get().warmModel !== settings.get().aiModelPath)}
        onTick={(_frame) => {
          syncInlinePreference(aiContext);
          pollModel(aiContext, modelUpdates);
        }}
      />
      <Shortcut
        command="debug-continue"
        keys="f5"
        onTrigger={() => {
          if (debuggerRunning.get()) sendDebugControl(continueControl, debuggerPaused, debugOutput, 'continue');
          else {
            mode.set('debug');
            sidebar.set(true);
            chooseDebugTarget(debugTarget);
          }
        }}
      />
      <Shortcut
        command="debug-stop"
        keys="shift-f5"
        onTrigger={() => {
          if (debuggerRunning.get()) sendDebugControl(stopControl, debuggerPaused, debugOutput, 'stop');
        }}
      />
      <Shortcut
        command="debug-stepOver"
        keys="f10"
        onTrigger={() => {
          sendDebugControl(debugControls, debuggerPaused, debugOutput, 'stepOver');
        }}
      />
      <Shortcut
        command="debug-stepIn"
        keys="f11"
        onTrigger={() => {
          sendDebugControl(stepIntoControl, debuggerPaused, debugOutput, 'stepIn');
        }}
      />
      <Shortcut
        command="debug-stepOut"
        keys="shift-f11"
        onTrigger={() => {
          sendDebugControl(stepOutControl, debuggerPaused, debugOutput, 'stepOut');
        }}
      />
      <Shortcut
        command="toggle-source-breakpoint"
        keys="f9"
        onTrigger={() => {
          if (mode.get() !== 'extensions') toggleCursorBreakpoint(workspace, recovery, sourceBreakpoints, debuggerRunning.get());
        }}
      />
      <Shortcut
        command="show-extensions"
        keys="secondary-shift-x"
        onTrigger={() => {
          mode.set('extensions');
          sidebar.set(true);
          modelPaths.set(availableModelPaths(settings.get()));
          controlInput('extension-search', 'focus');
        }}
      />
      {referenceControls}
      {SaveConflicts(workspace, recovery, font)}
      {ChatEditReview(aiState, workspace, recovery, font, aiStore)}
      {chrome}
      {fileSearch}
      {refreshExplorer}
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
  const path = unwrap(canonicalize(parameters.length > 0 ? clone(parameters[0]) : unwrap(currentDir())));
  const requests = new Channel<string>(1);
  const results = new Channel<CheckResult>(1);
  const replies = results.sender();
  const formattingRequests = new Channel<FormattingRequest>(1);
  const formattingResults = new Channel<FormattingResult>(1);
  const formattingReplies = formattingResults.sender();
  const configured = unwrap(get('TSRUST'));
  const program: string = configured ?? 'tsrust';
  let startupConfig = defaultConfig();
  let startupError = '';

  match (configPath()) {
    Err(error) => {
      startupError = clone(error);
    },
    Ok(path) => {
      match (loadConfig(path)) {
        Err(error) => {
          startupError = clone(error);
        },
        Ok(config) => {
          startupConfig = config;
        }
      }
    }
  }

  const languageMessages = new Channel<LanguageMessage>(8);
  const languageUpdates = new Channel<LanguageUpdate>(8);
  const languageReplies = languageUpdates.sender();
  const modelRequests = new Channel<ModelRequest>(8);
  const modelUpdates = new Channel<ModelUpdate>(8);
  const modelReplies = modelUpdates.sender();
  const searchRequests = new Channel<ContentSearchRequest>(1);
  const searchReplies = new Channel<ContentSearchReply>(1);
  const cancellations = new Channel<int>(1);
  const gitRequests = new Channel<GitRequest>(1);
  const gitReplies = new Channel<GitReply>(1);
  const gitSender = gitReplies.sender();
  const searchSender = searchReplies.sender();
  const downloadChannel = new Channel<string>(1);
  const downloadSender = downloadChannel.sender();
  const downloadReplies = new Channel<string>(1);
  const downloadResults = downloadReplies.sender();
  const debugRequests = new Channel<DebugCommand>(1);
  const debugResults = new Channel<DebugOutput>(16);
  const debugReplies = debugResults.sender();
  const view = App(
    clone(path),
    requests,
    results,
    formattingRequests,
    formattingResults,
    clone(program),
    languageMessages,
    languageUpdates,
    modelRequests,
    modelUpdates,
    searchRequests,
    cancellations,
    searchReplies,
    gitRequests.sender(),
    gitReplies,
    downloadSender,
    downloadReplies,
    debugRequests,
    debugResults,
    clone(startupConfig),
    startupError
  );
  const modelProgram = unwrap(get('TSRUST_STUDIO_MODEL_HOST')) ?? join(dirname(unwrap(currentExe())), 'tsrust-studio-model-host');
  const workers = studioWorkers(
    clone(path),
    program,
    requests,
    replies,
    formattingRequests,
    formattingReplies,
    languageMessages,
    languageReplies,
    modelProgram,
    modelRequests,
    modelReplies,
    searchRequests,
    searchSender,
    cancellations,
    gitRequests,
    gitSender,
    downloadChannel,
    downloadResults,
    debugRequests,
    debugReplies,
    startupConfig.languageServers
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
