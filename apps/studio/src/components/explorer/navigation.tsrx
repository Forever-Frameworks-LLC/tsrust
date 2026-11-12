import {
  Text,
  Command,
  CommandItem,
  CommandEmpty,
  Node,
  Signal,
  computed,
  t,
  controlInput
} from 'std:ui';

import { FileSearchState, SearchActionContext, openFileSearchResult } from '../../lib/explorer/search-actions.tsr';
import { Workspace, Entry } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { message, localizeError } from '../../lib/localization.tsr';

function FileSearchItem(file: ReadOnly<Entry>, root: ReadOnly<string>): Node {
  const path = clone(file.path);
  const label = file.path.slice(root.length + 1);
  return (
    <CommandItem value={path} label={label} icon="file" />
  );
}

export function FileSearch(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  search: Signal<FileSearchState>,
  mode: Signal<string>,
  sidebar: Signal<boolean>,
  font: Signal<int>
): Node {
  const textClass = computed<string>(() => 'size-full min-h-0 text-[' + font.get().toString() + 'px]');
  return (
    <Command
      inputId="file-search"
      className={textClass.get()}
      bordered={false}
      query={search.get().query}
      placeholder={message("Find a file by name or path")}
      onQuery={(query) => {
        const next = search.get();
        next.query = clone(query);
        search.set(next);
      }}
      onSelect={(path) => {
        const context: SearchActionContext = {
          workspace, recovery, fileSearch: search, navigationMode: mode, sidebarVisible: sidebar, allowedRoot: clone(workspace.get().root), allowRead: true, allowNavigation: true, focusEditor: true
        };

        match (openFileSearchResult(context, search.get().root, path)) {
          Ok(_) => {},
          Err(error) => {
            const next = workspace.get();
            next.error = localizeError(error);
            workspace.set(next);
          }
        }
      }}
      onCancel={() => {
        mode.set('files');
        controlInput(clone(workspace.get().active), 'focus');
      }}
    >
      {...search.get().files.map((file: ReadOnly<Entry>) => FileSearchItem(file, search.get().root))}
      <CommandEmpty>
        <Text>
          {t('No files found.')}
        </Text>
      </CommandEmpty>
    </Command>
  );
}
