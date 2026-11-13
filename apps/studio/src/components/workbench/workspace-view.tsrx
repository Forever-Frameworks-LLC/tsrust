import { Sender } from 'std:futures';
import {
  View,
  Button,
  Dock,
  Node,
  Signal,
  computed,
  signal,
  t
} from 'std:ui';

import { FileSearchState } from '../../lib/explorer/search-actions.tsr';
import { LanguageReferences } from '../../lib/language/references.tsr';
import { planCloseEditors } from '../../lib/workspace/close-editors.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { QueuedFormatting } from '../../lib/workspace/formatting-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, OpenFile, selectFile } from '../../lib/workspace/workspace.tsr';
import { LanguageState } from '../../services/lsp/controller.tsr';
import { LanguageMessage } from '../../services/lsp/session.tsr';
import { CloseEditorsState, CloseEditorsDialog, requestCloseEditors } from '../editor/close-editors.tsrx';
import { FilePanel } from '../editor/editor.tsrx';
import { openPalette } from '../explorer/navigation.tsrx';
import { closeTab } from './chrome.tsrx';

export function EditorWorkspace(
  workspace: Signal<Workspace>,
  font: Signal<int>,
  recovery: Signal<Recovery>,
  focusQueue: Signal<QueuedFormatting[]>,
  language: Signal<LanguageState>,
  messages: Sender<LanguageMessage>,
  references: Signal<LanguageReferences>,
  panel: Signal<string>,
  fileSearch: Signal<FileSearchState>
): Node {
  const renderFile = (file: ReadOnly<OpenFile>): Node => FilePanel(
    file,
    workspace,
    font,
    recovery,
    focusQueue,
    language,
    messages.share(),
    references,
    panel
  );
  const closing = signal<CloseEditorsState>({
    open: false, plan: planCloseEditors(workspace.get())
  });
  const layoutFailed = signal(false);
  const dockClass = computed<string>(() => 'size-full text-[' + font.get().toString() + 'px]');
  const initialLayout = clone(workspace.get().dockLayout);
  const retainLayout = (layout: ReadOnly<string>): void => {
    const next = workspace.get();

    if (!layoutFailed.get() && next.dockLayout !== layout) {
      next.dockLayout = clone(layout);
      publishWorkspace(next, workspace, recovery);
    }
  };
  const reportLayoutError = (error: ReadOnly<string>): void => {
    layoutFailed.set(true);
    const next = workspace.get();
    next.error = clone(error);
    workspace.set(next);
  };
  return (
    <View className="flex-col size-full min-w-0 min-h-0 bg-background">
      {CloseEditorsDialog(workspace, recovery, closing)}
      <View hidden={workspace.get().files.length !== 0} className="flex-col grow items-center justify-center">
        <Button
          icon="file-search"
          variant="ghost"
          onPress={() => {
            openPalette(workspace, recovery, fileSearch);
          }}
        >
          {t("Open a file")}
        </Button>
      </View>
      <View hidden={workspace.get().files.length === 0} className="flex-col grow min-h-0">
        <Dock
          initialLayout={initialLayout}
          onLayoutChange={retainLayout}
          onLayoutError={reportLayoutError}
          value={workspace.get().active}
          tabbed={true}
          className={dockClass.get()}
          onChange={(path) => {
            const next = workspace.get();

            if (next.active !== path && selectFile(next, clone(path))) publishWorkspace(next, workspace, recovery);
          }}
          onCloseAll={() => {
            requestCloseEditors(workspace, recovery, closing);
          }}
          onClose={(path) => {
            closeTab(workspace, recovery, clone(path));
          }}
        >
          {...workspace.get().files.map(renderFile)}
        </Dock>
      </View>
    </View>
  );
}
