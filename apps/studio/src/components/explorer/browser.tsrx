import { basename } from 'std:path';
import { platform } from 'std:os';
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
  TreeMenu,
  TreeBackgroundMenu,
  MenuItem,
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogAction,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  Input,
  revealPath,
  translate,
  Signal,
  computed
} from 'std:ui';

import { changeExplorerEntry } from '../../lib/explorer/file-operations.tsr';
import { QueuedFormatting } from '../../lib/workspace/formatting-actions.tsr';
import { queueSaves, unsavedTargets } from '../../services/save-actions.tsr';
import { fileDecoration } from '../../lib/explorer/file-icons.tsr';
import { requestTreeRefresh } from '../../lib/explorer/tree-actions.tsr';
import { TreeSession, refreshTree, pollTree } from '../../lib/explorer/tree-controller.tsr';
import { DirectoryTree } from '../../lib/explorer/tree.tsr';
import { localizeError, message, accessibleName } from '../../lib/localization.tsr';
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
  const named = directory && children.length === 0 ? 'folder' : clone(decoration.icon);
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

function beginExplorerAction(
  target: Signal<string>,
  action: Signal<string>,
  name: Signal<string>,
  problem: Signal<string>,
  command: string,
  item: string
): void {
  target.set(clone(item));
  name.set(command === 'Rename' ? basename(item) : '');
  problem.set('');
  action.set(command);
}

function applyExplorerAction(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  revision: Signal<int>,
  action: Signal<string>,
  error: Signal<string>,
  command: string,
  item: string,
  value: string
): void {
  const next = workspace.get();

  match (changeExplorerEntry(next, command, item, value)) {
    Ok(_) => {
      next.error = '';
      publishWorkspace(next, workspace, recovery);
      action.set('');
      revision.set(revision.get() + 1);
    },
    Err(reason) => {
      error.set(reason);
    }
  }
}

export function ProjectTree(
  path: string,
  workspace: Signal<Workspace>,
  font: Signal<int>,
  recovery: Signal<Recovery>,
  settings: Signal<StudioConfig>,
  revision: Signal<int>,
  snapshot: Signal<DirectoryTree>,
  queue: Signal<QueuedFormatting[]>
): Node {
  const textClass = computed<string>(() => 'font-mono text-[' + font.get().toString() + 'px]');
  const dialogText = computed<string>(() => 'text-[' + font.get().toString() + 'px]');
  const dialogButton = computed<string>(() => 'rounded-md px-3 py-1 hover:bg-accent ' + dialogText.get());
  const treeClass = computed<string>(() => 'grow min-h-0 min-w-0 ' + textClass.get());
  const error = signal('');
  const appliedRevision = signal(-1);
  const appliedPolicy = signal(settings.get().treeIncludeIgnored);
  const session: TreeSession = {
    root: clone(path), workspace, settings, revision, appliedRevision, appliedPolicy, snapshot, error
  };
  const target = signal(clone(path));
  const action = signal('');
  const name = signal('');
  const problem = signal('');
  const reveal = platform() === 'darwin' ? 'Reveal in Finder' : platform() === 'win32' ? 'Reveal in File Explorer' : 'Reveal in file manager';
  const confirm = () => {
    applyExplorerAction(
      workspace,
      recovery,
      revision,
      action,
      problem,
      action.get(),
      target.get(),
      name.get()
    );
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
        backgroundDropTarget={clone(path)}
        onContextMenu={(item) => {
          target.set(item);
        }}
        onItemDrop={(items) => {
          const pair = JSON.parse<string[]>(items);

          if (pair.length === 2) applyExplorerAction(
            workspace,
            recovery,
            revision,
            action,
            error,
            'Move',
            clone(pair[0]),
            clone(pair[1])
          );
        }}
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
        <TreeItem id={clone(path)} label={basename(path) === '' ? clone(path) : basename(path)} defaultExpanded={true} icon="folder-open">
          {...directoryItems(0, snapshot.get(), workspace, settings)}
        </TreeItem>
        <TreeMenu>
          <MenuItem
            label={message('New file')}
            onSelect={() => {
              beginExplorerAction(
                target,
                action,
                name,
                problem,
                'New file',
                target.get()
              );
            }}
          />
          <MenuItem
            label={message('New folder')}
            onSelect={() => {
              beginExplorerAction(
                target,
                action,
                name,
                problem,
                'New folder',
                target.get()
              );
            }}
          />
          <MenuItem
            label={message('Rename')}
            onSelect={() => {
              beginExplorerAction(
                target,
                action,
                name,
                problem,
                'Rename',
                target.get()
              );
            }}
          />
          <MenuItem
            label={message('Delete')}
            onSelect={() => {
              beginExplorerAction(
                target,
                action,
                name,
                problem,
                'Delete',
                target.get()
              );
            }}
          />
          <MenuItem
            label={message('Save')}
            onSelect={() => {
              const item = target.get();
              const targets = unsavedTargets(workspace.get()).filter((entry) => entry.path === item);

              match (queueSaves({
                workspace, queue, allowSave: true
              }, targets)) {
                Ok(_) => {},
                Err(reason) => {
                  error.set(reason);
                }
              }
            }}
          />
          <MenuItem
            label={message(reveal)}
            onSelect={() => {
              revealPath(target.get());
            }}
          />
        </TreeMenu>
        <TreeBackgroundMenu>
          <MenuItem
            label={message('New file')}
            onSelect={() => {
              beginExplorerAction(
                target,
                action,
                name,
                problem,
                'New file',
                clone(workspace.get().root)
              );
            }}
          />
          <MenuItem
            label={message('New folder')}
            onSelect={() => {
              beginExplorerAction(
                target,
                action,
                name,
                problem,
                'New folder',
                clone(workspace.get().root)
              );
            }}
          />
        </TreeBackgroundMenu>
      </Tree>
      <Dialog
        open={action.get() !== ''}
        width={420}
        onOk={confirm}
        onOpenChange={(opened) => {
          if (!opened) action.set('');
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Text className={dialogText.get() + ' font-semibold'}>
                {translate(action.get())}
              </Text>
            </DialogTitle>
            <DialogDescription>
              <Text className={dialogText.get() + ' text-muted-foreground'}>
                {target.get()}
              </Text>
            </DialogDescription>
          </DialogHeader>
          <View hidden={action.get() === 'Delete'}>
            <Input
              className={dialogText.get()}
              value={name.get()}
              aria-label={accessibleName('Name')}
              onChange={(event) => {
                name.set(clone(event.target.value));
              }}
            />
          </View>
          <View hidden={action.get() !== 'Delete'}>
            <Text className={dialogText.get()}>
              {translate('Permanently delete this item and any contents?')}
            </Text>
          </View>
          <View hidden={problem.get() === ''}>
            <Text role="status" className={dialogText.get() + ' text-destructive'}>
              {localizeError(problem.get())}
            </Text>
          </View>
          <DialogFooter>
            <Button
              className={dialogButton.get()}
              onPress={() => {
                action.set('');
              }}
            >
              {translate('Cancel')}
            </Button>
            <DialogAction>
              <Button className={dialogButton.get() + ' bg-accent'}>
                {translate(action.get())}
              </Button>
            </DialogAction>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </View>
  );
}
