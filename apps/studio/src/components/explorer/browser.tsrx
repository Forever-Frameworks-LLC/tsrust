import { stat } from 'std:fs';
import {
  Node,
  View,
  Shortcut,
  Ticker,
  signal,
  Text,
  Tree,
  TreeItem,
  Signal,
  computed
} from 'std:ui';

import { TreeSession, refreshTree, pollTree } from '../../lib/explorer/tree-controller.tsr';
import { DirectoryTree } from '../../lib/explorer/tree.tsr';
import { requestTreeRefresh } from '../../lib/explorer/tree-actions.tsr';
import { fileDecoration } from '../../lib/explorer/file-icons.tsr';
import { localizeError } from '../../lib/localization.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { dirty } from '../../lib/workspace/documents.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, openFile } from '../../lib/workspace/workspace.tsr';

function FileTreeItem(
  path: string,
  name: string,
  directory: boolean,
  children: Node[],
  workspace: Signal<Workspace>,
  settings: Signal<StudioConfig>
): Node {
  const decoration = fileDecoration(name, directory);
  const labelPath = clone(path);
  const label = computed<string>(() => name + (workspace.get().files.some((file) => file.document.path === labelPath && dirty(file.document)) ? ' ●' : ''));
  const glyph = clone(decoration.svg);
  const tint = clone(decoration.color);
  const named = clone(decoration.icon);
  const svg = computed<string>(() => settings.get().fileIcons ? clone(glyph) : '');
  const color = computed<string>(() => settings.get().fileIcons ? clone(tint) : '');
  const icon = computed<string>(() => settings.get().fileIcons ? clone(named) : '');
  return (
    <TreeItem id={path} label={label.get()} iconSvg={svg.get()} iconColor={color.get()} icon={icon.get()}>
      {...children}
    </TreeItem>
  );
}

function directoryItems(position: int, tree: ReadOnly<DirectoryTree>, workspace: Signal<Workspace>, settings: Signal<StudioConfig>): Node[] {
  const nodes: Node[] = [];

  if (position < 0 || position >= tree.directories.length) return nodes;

  for (const entry of tree.directories[position].children) {
    const nested = entry.directory ? directoryItems(entry.directoryIndex, tree, workspace, settings) : [];
    nodes.push(FileTreeItem(
      clone(entry.path),
      clone(entry.name),
      entry.directory,
      nested,
      workspace,
      settings
    ));
  }

  return nodes;
}

export function RefreshExplorer(path: string, workspace: Signal<Workspace>, recovery: Signal<Recovery>, revision: Signal<int>): Node {
  return (
    <Shortcut
      command="refresh-explorer"
      keys="secondary-alt-r"
      onTrigger={() => {
        const next = workspace.get();

        match (requestTreeRefresh(revision, clone(path), clone(path), clone(next.root), true)) {
          Ok(_) => {},
          Err(error) => {
            next.error = error;
            publishWorkspace(next, workspace, recovery);
          }
        }
      }}
    />
  );
}

export function ProjectTree(
  path: string,
  workspace: Signal<Workspace>,
  font: Signal<int>,
  recovery: Signal<Recovery>,
  settings: Signal<StudioConfig>,
  revision: Signal<int>
): Node {
  const textClass = computed<string>(() => 'font-mono text-[' + font.get().toString() + 'px]');
  const treeClass = computed<string>(() => 'grow min-h-0 min-w-0 ' + textClass.get());
  const snapshot = signal<DirectoryTree>({
    root: clone(path), directories: []
  });
  const error = signal('');
  const appliedRevision = signal(-1);
  const appliedPolicy = signal(settings.get().treeIncludeIgnored);
  const session: TreeSession = {
    root: clone(path), workspace, settings, revision, appliedRevision, appliedPolicy, snapshot, error
  };
  refreshTree(session);
  const errorText = computed<string>(() => localizeError(error.get()));
  return (
    <View className="flex-col size-full min-h-0">
      <Ticker
        interval={150}
        onTick={(_frame) => {
          pollTree(session);
        }}
      />
      <View hidden={errorText.get() === ''} className="shrink-0">
        <Text className={textClass.get() + ' text-destructive px-2 py-1'}>
          {errorText.get()}
        </Text>
      </View>
      <Tree
        defaultSelected={workspace.get().active}
        className={treeClass.get()}
        onSelect={(selectedPath) => {
          const next = workspace.get();

          match (stat(selectedPath)) {
            Ok(metadata) => {
              if (metadata.isDirectory) return;
            },
            Err(error) => {
              next.error = clone(error.message);
              publishWorkspace(next, workspace, recovery);
              return;
            }
          }

          match (openFile(next, selectedPath)) {
            Ok(_) => {},
            Err(message) => {
              next.error = message;
            }
          }

          publishWorkspace(next, workspace, recovery);
        }}
      >
        {...directoryItems(0, snapshot.get(), workspace, settings)}
      </Tree>
    </View>
  );
}
