import {
  View,
  Text,
  Input,
  Button,
  Node,
  Signal,
  signal,
  computed
} from 'std:ui';

import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace, Entry, projectFiles, openFile } from '../../lib/workspace/workspace.tsr';


function SearchResult(
  file: ReadOnly<Entry>,
  root: string,
  workspace: Signal<Workspace>,
  query: Signal<string>,
  font: Signal<int>,
  recovery: Signal<Recovery>
): Node {
  const path = clone(file.path);
  const label = file.path.slice(root.length + 1);
  const searchText = label.toLowerCase();
  const resultClass = computed<string>(() => 'w-full flex-row items-center justify-start px-2 py-1 rounded-sm hover:bg-accent text-[' + font.get().toString() + 'px]');
  return (
    <View hidden={!searchText.includes(query.get().toLowerCase())} className="flex-col">
      <Button
        className={resultClass.get()}
        onPress={() => {
          const next = workspace.get();

          match (openFile(next, path)) {
            Ok(_) => {},
            Err(message) => {
              next.error = clone(message);
            }
          }

          publishWorkspace(next, workspace, recovery);
        }}
      >
        <Text className="truncate">
          {label}
        </Text>
      </Button>
    </View>
  );
}

export function FileSearch(root: string, workspace: Signal<Workspace>, font: Signal<int>, recovery: Signal<Recovery>): Node {
  const query = signal('');
  const inputClass = computed<string>(() => 'w-full min-w-0 text-[' + font.get().toString() + 'px]');

  match (projectFiles(root)) {
    Err(message) => {
      const errorText = computed<string>(() => localizeError(message));
      return (
        <Text>
          {errorText.get()}
        </Text>
      );
    },
    Ok(files) => {
      const results: Node[] = [];

      for (const file of files) results.push(SearchResult(
        file,
        clone(root),
        workspace,
        query,
        font,
        recovery
      ));
      return (
        <View className="flex-col size-full p-3 gap-2">
          <Input
            className={inputClass.get()}
            value={query.get()}
            placeholder={message("Find a file by name or path")}
            aria-label={accessibleName("Find project file")}
            onChange={(event) => {
              query.set(clone(event.target.value));
            }}
          />
          <View className="flex-col grow min-h-0 overflow-y-auto">
            {...results}
          </View>
        </View>
      );
    }
  }
}
