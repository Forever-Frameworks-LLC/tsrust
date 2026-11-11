import { stat } from 'std:fs';
import {
  Node,
  Text,
  Tree,
  TreeItem,
  Signal,
  computed
} from 'std:ui';

import { fileDecoration } from '../../lib/explorer/file-icons.tsr';
import { localizeError } from '../../lib/localization.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { dirty } from '../../lib/workspace/documents.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, entries, openFile } from '../../lib/workspace/workspace.tsr';

function FileTreeItem(path: string, name: string, directory: boolean, children: Node[], workspace: Signal<Workspace>, settings: Signal<StudioConfig>): Node {
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

function directoryItems(path: string, workspace: Signal<Workspace>, settings: Signal<StudioConfig>): Result<Node[], string> {
  match (entries(path)) {
    Err(message) => {
      return Err(clone(message));
    },
    Ok(children) => {
      const nodes: Node[] = [];

      for (const entry of children) {
        let nested: Node[] = [];

        if (entry.directory) {
          match (directoryItems(entry.path, workspace, settings)) {
            Ok(items) => {
              nested = items;
            },
            Err(message) => {
              return Err(clone(message));
            }
          }
        }

        nodes.push(FileTreeItem(clone(entry.path), clone(entry.name), entry.directory, nested, workspace, settings));
      }

      return Ok(nodes);
    }
  }
}

export function ProjectTree(path: string, workspace: Signal<Workspace>, font: Signal<int>, recovery: Signal<Recovery>, settings: Signal<StudioConfig>): Node {
  const treeClass = computed<string>(() => 'size-full min-w-0 font-mono text-[' + font.get().toString() + 'px]');

  match (directoryItems(path, workspace, settings)) {
    Err(message) => {
      const errorText = computed<string>(() => localizeError(message));
      return (
        <Text>
          {errorText.get()}
        </Text>
      );
    },
    Ok(children) => {
      return (
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
                next.error = clone(message);
              }
            }

            publishWorkspace(next, workspace, recovery);
          }}
        >
          {...children}
        </Tree>
      );
    }
  }
}
