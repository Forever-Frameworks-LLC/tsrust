import {
  View,
  Text,
  Dialog,
  DialogContent,
  Command,
  CommandItem,
  CommandEmpty,
  Node,
  Signal,
  computed,
  t
} from 'std:ui';

import { FileSearchState, SearchActionContext, showFileSearch, closeFileSearch, openFileSearchResult } from '../../lib/explorer/search-actions.tsr';
import { PaletteCommand, paletteCommands } from '../../lib/workspace/commands.tsr';
import { Workspace, Entry } from '../../lib/workspace/workspace.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { message } from '../../lib/localization.tsr';

export function openPalette(workspace: Signal<Workspace>, recovery: Signal<Recovery>, search: Signal<FileSearchState>, commands: boolean = false): void {
  const context: SearchActionContext = {
    workspace, recovery, fileSearch: search, allowedRoot: clone(workspace.get().root), allowRead: true, allowNavigation: true, allowModal: true, focusEditor: true
  };

  match (showFileSearch(context, workspace.get().root, commands ? '>' : undefined)) {
    Ok(_) => {},
    Err(error) => {
      const next = workspace.get();
      next.error = error;
      workspace.set(next);
    }
  }
}

function closePalette(workspace: Signal<Workspace>, recovery: Signal<Recovery>, search: Signal<FileSearchState>): void {
  const context: SearchActionContext = {
    workspace, recovery, fileSearch: search, allowedRoot: clone(workspace.get().root), allowNavigation: true, focusEditor: false
  };

  match (closeFileSearch(context, workspace.get().root)) {
    Ok(_) => {},
    Err(error) => {
      const next = workspace.get();
      next.error = error;
      workspace.set(next);
    }
  }
}

function FileSearchItem(file: ReadOnly<Entry>, root: ReadOnly<string>): Node {
  const path = clone(file.path);
  const label = file.path.slice(root.length + 1);
  return (
    <CommandItem value={path} label={label} icon="file" />
  );
}

function CommandItemRow(command: ReadOnly<PaletteCommand>): Node {
  const id = clone(command.id);
  const value = 'command:' + command.id;
  const label = clone(command.label);
  const disabled = command.disabled;
  return (
    <CommandItem value={value} command={id} label={label} disabled={disabled} icon="command" />
  );
}

function fileMatches(search: ReadOnly<FileSearchState>): Entry[] {
  if (search.query.startsWith('>')) return [];
  const query = search.query.toLowerCase();
  return search.files.filter((file) => file.path.slice(search.root.length + 1).toLowerCase().includes(query));
}

export function FileSearch(workspace: Signal<Workspace>, recovery: Signal<Recovery>, search: Signal<FileSearchState>, font: Signal<int>): Node {
  const textClass = computed<string>(() => 'w-full min-h-0 text-[' + font.get().toString() + 'px]');
  return (
    <Dialog
      open={search.get().open ?? false}
      width={640}
      maxWidth={720}
      marginTop={64}
      closeButton={false}
      overlay={false}
      onOpenChange={(open) => {
        if (!open) closePalette(workspace, recovery, search);
      }}
    >
      <DialogContent className="p-0">
        <View hidden={!(search.get().open ?? false)} className="w-full min-h-0">
          <Command
            inputId="file-search"
            className={textClass.get()}
            maxHeight={360}
            bordered={false}
            filterable={false}
            query={search.get().query}
            placeholder={message("Search files or type > for commands")}
            onQuery={(query) => {
              const next = search.get();
              next.query = clone(query);
              search.set(next);
            }}
            onSelect={(path) => {
              if (path.startsWith('command:')) {
                closePalette(workspace, recovery, search);
                return;
              }

              const context: SearchActionContext = {
                workspace, recovery, fileSearch: search, allowedRoot: clone(workspace.get().root), allowRead: true, allowNavigation: true, focusEditor: true
              };

              match (openFileSearchResult(context, search.get().root, path)) {
                Ok(_) => {},
                Err(error) => {
                  const next = workspace.get();
                  next.error = error;
                  workspace.set(next);
                }
              }
            }}
            onCancel={() => {
              closePalette(workspace, recovery, search);
            }}
          >
            {...fileMatches(search.get()).map((file: ReadOnly<Entry>) => FileSearchItem(file, search.get().root))}
            {...paletteCommands(workspace.get(), search.get().query).map((command: ReadOnly<PaletteCommand>) => CommandItemRow(command))}
            <CommandEmpty>
              <Text>
                {t('No matching files or commands.')}
              </Text>
            </CommandEmpty>
          </Command>
        </View>
      </DialogContent>
    </Dialog>
  );
}
