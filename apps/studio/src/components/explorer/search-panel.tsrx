import {
  View,
  Text,
  Input,
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  Button,
  Node,
  Signal,
  signal,
  computed,
  translateWith
} from 'std:ui';

import { SearchActionContext, runContentSearch, openSearchResult } from '../../lib/explorer/search-actions.tsr';
import { SearchHit, searchPreview } from '../../lib/explorer/search.tsr';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';

function ResultRow(hit: ReadOnly<SearchHit>, root: string, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  const rowClass = computed<string>(() => 'flex-col items-stretch justify-center gap-1 w-full min-w-0 shrink-0 px-3 py-2 rounded-none hover:bg-accent min-h-[' + (font.get() * 3 + 16).toString() + 'px] text-[' + font.get().toString() + 'px]');
  const parts = searchPreview(hit);
  const location = hit.path.slice(root.length + 1) + ':' + hit.line.toString();
  const label = clone(location) + ' ' + parts.map((part) => clone(part.text)).join('');
  const target = clone(hit);
  const context: SearchActionContext = {
    workspace, recovery, allowNavigation: true, focusEditor: true
  };
  return (
    <View className="flex-col shrink-0 border-b border-border">
      <Button
        variant="ghost"
        className={rowClass.get()}
        aria-label={label}
        onPress={() => {
          match (openSearchResult(context, target)) {
            Ok(_) => {},
            Err(message) => {
              const next = workspace.get();
              next.error = clone(message);
              publishWorkspace(next, workspace, recovery);
            }
          }
        }}
      >
        <Text className="w-full truncate text-muted-foreground">
          {hit.path.slice(root.length + 1)}
        </Text>
        <Text className="w-full truncate font-mono">
          <Text className="text-muted-foreground">
            {hit.line.toString() + '  '}
          </Text>
          {...parts.map((part) => <Text className={part.matched ? 'bg-primary/20 text-primary font-semibold' : ''}>
            {clone(part.text)}
          </Text>
          )}
        </Text>
      </Button>
    </View>
  );
}

interface ContentSearchState {
  query: string;
  regex: boolean;
  sensitive: boolean;
  hits: SearchHit[];
  error: string;
  skipped: int;
  searched: boolean;
}

function submitContentSearch(workspace: Signal<Workspace>, recovery: Signal<Recovery>, settings: Signal<StudioConfig>, search: Signal<ContentSearchState>): void {
  const next = search.get();
  const context: SearchActionContext = {
    workspace, recovery, allowNavigation: false, focusEditor: false, allowRead: true, allowedRoot: clone(workspace.get().root)
  };

  match (runContentSearch(context, clone(next.query), next.regex, next.sensitive, settings.get().searchIncludeIgnored)) {
    Ok(result) => {
      next.hits = result.hits;
      next.skipped = result.skipped;
      next.error = '';
    },
    Err(error) => {
      next.hits = [];
      next.skipped = 0;
      next.error = error;
    }
  }

  next.searched = next.query !== '';
  search.set(next);
}

export function ContentSearch(workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>, settings: Signal<StudioConfig>): Node {
  const fontClass = computed<string>(() => 'text-[' + font.get().toString() + 'px]');
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 p-2 gap-2 text-[' + font.get().toString() + 'px]');
  const search = signal<ContentSearchState>({
    query: '', regex: false, sensitive: false, hits: [], error: '', skipped: 0, searched: false
  });
  const summary = computed<string>(() => translateWith('matchingLines', {
    count: search.get().hits.length
  }) + (search.get().skipped > 0 ? ' • ' + translateWith('skippedFiles', {
    count: search.get().skipped
  }) : ''));
  return (
    <View className={panelClass.get()}>
      <View className="shrink-0 h-8 min-w-0">
        <InputGroup size="small" className="w-full h-full min-w-0">
          <Input
            id="content-search"
            value={search.get().query}
            placeholder={message("Search file contents")}
            aria-label={accessibleName("Search file contents")}
            className={fontClass.get()}
            onChange={(event) => {
              const next = search.get();
              next.query = clone(event.target.value);
              next.searched = false;
              next.hits = [];
              next.error = '';
              search.set(next);
            }}
            onSubmit={(_secondary) => {
              submitContentSearch(workspace, recovery, settings, search);
            }}
          />
          <InputGroupAddon align="inline-end">
            <InputGroupButton
              icon="case-sensitive"
              selected={search.get().sensitive}
              tooltip={message("Match case")}
              accessibilityLabel={message("Match case")}
              onClick={() => {
                const next = search.get();
                next.sensitive = !next.sensitive;
                search.set(next);
                submitContentSearch(workspace, recovery, settings, search);
              }}
            />
            <InputGroupButton
              icon="regex"
              selected={search.get().regex}
              tooltip={message("Use regular expression")}
              accessibilityLabel={message("Use regular expression")}
              onClick={() => {
                const next = search.get();
                next.regex = !next.regex;
                search.set(next);
                submitContentSearch(workspace, recovery, settings, search);
              }}
            />
            <InputGroupButton
              icon="search"
              tooltip={message("Run content search")}
              accessibilityLabel={message("Run content search")}
              onClick={() => {
                submitContentSearch(workspace, recovery, settings, search);
              }}
            />
          </InputGroupAddon>
        </InputGroup>
      </View>
      <View hidden={!search.get().searched || search.get().error !== ''} className="shrink-0 px-1">
        <Text>
          {summary.get()}
        </Text>
      </View>
      <View hidden={search.get().error === ''} className="shrink-0 px-1">
        <Text className="text-destructive">
          {localizeError(search.get().error)}
        </Text>
      </View>
      <View className="flex-col grow min-h-0 overflow-y-auto">
        {...search.get().hits.map((hit: ReadOnly<SearchHit>) => ResultRow(hit, workspace.get().root, workspace, recovery, font))}
      </View>
    </View>
  );
}
