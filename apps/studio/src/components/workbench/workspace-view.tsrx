import { View, Text, Button, Dock, Node, Signal, computed, signal, t } from 'std:ui';
import { Workspace, OpenFile, selectFile } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { FilePanel } from '../editor/editor.tsrx';
import { closeTab } from './chrome.tsrx';
import { localizeError } from '../../lib/localization.tsr';

export function EditorWorkspace(workspace: Signal<Workspace>, font: Signal<int>, recovery: Signal<Recovery>, mode: Signal<string>, sidebar: Signal<boolean>): Node {
  const layoutFailed = signal(false);
  const dockClass = computed<string>(() => 'size-full text-[' + font.get().toString() + 'px]');
  const initialLayout = clone(workspace.get().dockLayout);
  const error = computed<string>(() => localizeError(workspace.get().error));
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
  return <View className="flex-col size-full min-w-0 min-h-0 bg-background">
                  <View hidden={workspace.get().files.length !== 0} className="flex-col grow items-center justify-center">
                    <Button icon="file-search" variant="ghost" onPress={() => { mode.set('filesearch'); sidebar.set(true); }}>{t("Open a file")}</Button>
                  </View>
                  <View hidden={workspace.get().files.length === 0} className="flex-col grow min-h-0">
                    <Dock initialLayout={initialLayout} onLayoutChange={retainLayout} onLayoutError={reportLayoutError} value={workspace.get().active} tabbed={true} className={dockClass.get()} onChange={(path) => { const next = workspace.get(); if (next.active !== path && selectFile(next, clone(path))) publishWorkspace(next, workspace, recovery); }} onClose={(path) => { closeTab(workspace, recovery, clone(path)); }}>
                      {...workspace.get().files.map((file: ReadOnly<OpenFile>) => FilePanel(file, workspace, font, recovery))}
                    </Dock>
                  </View>
                  <View hidden={workspace.get().error === ''} className="px-3 py-2"><Text className="text-destructive">{error.get()}</Text></View>
                </View>;
}
