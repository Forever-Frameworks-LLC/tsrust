import { Node, Text, Tree, TreeItem, Signal, computed } from 'std:ui';
import { stat } from 'std:fs';
import { Workspace, entries, openFile } from './workspace.tsr';

function directoryItems(path: string): Result<Node[], string> {
  match (entries(path)) {
    Err(message) => {
      return Err(clone(message));
    },
    Ok(children) => {
      const nodes: Node[] = [];
      for (const entry of children) {
        let nested: Node[] = [];
        if (entry.directory) {
          match (directoryItems(entry.path)) {
            Ok(items) => {
              nested = items;
            },
            Err(message) => {
              return Err(clone(message));
            }
          }
        }

        nodes.push(<TreeItem id={clone(entry.path)} label={clone(entry.name)}>{...nested}</TreeItem>);
      }

      return Ok(nodes);
    }
  }
}

export function ProjectTree(path: string, workspace: Signal<Workspace>, font: Signal<int>): Node {
  const treeClass = computed<string>(() => 'size-full min-w-0 font-mono text-[' + font.get().toString() + 'px]');
  match (directoryItems(path)) {
    Err(message) => {
      return <Text>{message}</Text>;
    },
    Ok(children) => {
      return <Tree defaultSelected={workspace.get().active} className={treeClass.get()} onSelect={(selectedPath) => {
        const next = workspace.get();
        match (stat(selectedPath)) {
          Ok(metadata) => { if (metadata.isDirectory) return; },
          Err(error) => {
            next.error = clone(error.message);
            workspace.set(next);
            return;
          }
        }
        match (openFile(next, selectedPath)) {
          Ok(_) => {},
          Err(message) => { next.error = clone(message); }
        }
        workspace.set(next);
      }}>
        {...children}
      </Tree>;
    }
  }
}
