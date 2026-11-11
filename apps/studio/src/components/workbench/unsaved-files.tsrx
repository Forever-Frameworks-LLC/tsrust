import {
  View,
  Button,
  DropdownMenu,
  MenuItem,
  Node,
  Signal,
  computed,
  translateWith
} from 'std:ui';

import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { dirty } from '../../lib/workspace/documents.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, OpenFile, selectFile } from '../../lib/workspace/workspace.tsr';

function UnsavedItem(file: ReadOnly<OpenFile>, workspace: Signal<Workspace>, recovery: Signal<Recovery>): Node {
  const path = clone(file.document.path);
  const label = file.document.path.slice(workspace.get().root.length + 1) + ' ●';
  return (
    <MenuItem
      label={label}
      onSelect={() => {
        const next = workspace.get();

        if (selectFile(next, clone(path))) publishWorkspace(next, workspace, recovery);
      }}
    />
  );
}

export function UnsavedFiles(workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  const fontClass = computed<string>(() => 'gap-1 text-[' + font.get().toString() + 'px]');
  const count = computed<int>(() => workspace.get().files.filter((file) => dirty(file.document)).length);
  const label = computed<string>(() => translateWith('Unsaved files: {count}', {
    count: count.get()
  }));
  return (
    <View hidden={count.get() === 0}>
      <DropdownMenu fontSize={font.get()}>
        <Button icon="file-pen" variant="ghost" size="xsmall" className={fontClass.get()} tooltip={label.get()} aria-label={label.get()}>
          {count.get().toString()}
        </Button>
        {...workspace.get().files.filter((file) => dirty(file.document)).map((file: ReadOnly<OpenFile>) => UnsavedItem(file, workspace, recovery))}
      </DropdownMenu>
    </View>
  );
}
