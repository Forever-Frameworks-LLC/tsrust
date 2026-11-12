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
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
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
        tooltip={location}
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

export function ContentSearch(workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>, settings: Signal<StudioConfig>): Node {
  const fontClass = computed<string>(() => 'text-[' + font.get().toString() + 'px]');
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 p-2 gap-2 text-[' + font.get().toString() + 'px]');
  const allowedRoot = clone(workspace.get().root);
  const context: SearchActionContext = {
    workspace, recovery, allowNavigation: false, focusEditor: false, allowRead: true, allowedRoot
  };
  const query = signal('');
  const regex = signal(false);
  const sensitive = signal(false);
  const hits = signal<SearchHit[]>([]);
  const error = signal('');
  const skipped = signal(0);
  const summary = computed<string>(() => translateWith('matchingLines', {
    count: hits.get().length
  }) + ' • ' + translateWith('skippedFiles', {
    count: skipped.get()
  }));
  return (
    <View className={panelClass.get()}>
      <InputGroup size="small" className="w-full min-w-0">
        <Input
          value={query.get()}
          placeholder={message("Search file contents")}
          aria-label={accessibleName("Search file contents")}
          className={fontClass.get()}
          onChange={(event) => {
            query.set(clone(event.target.value));
          }}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            icon="case-sensitive"
            selected={sensitive.get()}
            tooltip={message("Match case")}
            accessibilityLabel={message("Match case")}
            onClick={() => {
              sensitive.set(!sensitive.get());
            }}
          />
          <InputGroupButton
            icon="regex"
            selected={regex.get()}
            tooltip={message("Use regular expression")}
            accessibilityLabel={message("Use regular expression")}
            onClick={() => {
              regex.set(!regex.get());
            }}
          />
          <InputGroupButton
            icon="search"
            tooltip={message("Run content search")}
            accessibilityLabel={message("Run content search")}
            onClick={() => {
              match (runContentSearch(context, query.get(), regex.get(), sensitive.get(), settings.get().searchIncludeIgnored)) {
                Ok(result) => {
                  hits.set(result.hits);
                  skipped.set(result.skipped);
                  error.set('');
                },
                Err(message) => {
                  hits.set([]);
                  skipped.set(0);
                  error.set(message);
                }
              }
            }}
          />
        </InputGroupAddon>
      </InputGroup>
      <Text>
        {summary.get()}
      </Text>
      <View hidden={error.get() === ''}>
        <Text className="text-destructive">
          {localizeError(error.get())}
        </Text>
      </View>
      <View className="flex-col grow min-h-0 overflow-y-auto">
        {...hits.get().map((hit: ReadOnly<SearchHit>) => ResultRow(hit, workspace.get().root, workspace, recovery, font))}
      </View>
    </View>
  );
}
