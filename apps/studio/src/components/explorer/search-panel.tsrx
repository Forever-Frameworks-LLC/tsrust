import { Channel, Sender } from 'std:futures';
import {
  Ticker,
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

import { SearchActionContext, openSearchResult } from '../../lib/explorer/search-actions.tsr';
import { SearchHit, searchPreview } from '../../lib/explorer/search.tsr';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';

import { ContentSearchRequest, ContentSearchReply } from '../../services/content-search.tsr';

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
  generation: int;
  pending: boolean;
  running: boolean;
}

function submitContentSearch(search: Signal<ContentSearchState>): void {
  const next = search.get();
  next.generation += 1;
  next.pending = next.query !== '';
  next.searched = false;
  next.hits = [];
  next.error = '';
  search.set(next);
}

function pollContentSearch(workspace: Signal<Workspace>, settings: Signal<StudioConfig>, search: Signal<ContentSearchState>, requests: ReadOnly<Sender<ContentSearchRequest>>, replies: Mutable<Channel<ContentSearchReply>>): void {
  const next = search.get();

  if (next.running) {
    match (replies.tryReceive()) {
      Ok(reply) => {
        next.running = false;

        if (next.generation === reply.generation) {
          next.hits = reply.result.hits;
          next.skipped = reply.result.skipped;
          next.error = reply.error;
          next.searched = true;
        }
      },
      Err(error) => {
        if (error.code === 'empty') return;
        next.running = false;
        next.pending = false;
        next.error = clone(error.code);
      }
    }
  }

  if (next.pending) {
    next.pending = false;

    match (requests.trySend({
      generation: next.generation, workspace: workspace.get(), query: clone(next.query), regex: next.regex, sensitive: next.sensitive, includeIgnored: settings.get().searchIncludeIgnored
    })) {
      Ok(_) => {
        next.running = true;
      },
      Err(error) => {
        next.error = clone(error.code);
      }
    }
  }

  search.set(next);
}

export function ContentSearch(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  font: Signal<int>,
  settings: Signal<StudioConfig>,
  requests: ReadOnly<Channel<ContentSearchRequest>>,
  replies: Channel<ContentSearchReply>
): Node {
  const sender = requests.sender();
  const fontClass = computed<string>(() => 'text-[' + font.get().toString() + 'px]');
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 p-2 gap-2 text-[' + font.get().toString() + 'px]');
  const search = signal<ContentSearchState>({
    query: '', regex: false, sensitive: false, hits: [], error: '', skipped: 0, searched: false, generation: 0, pending: false, running: false
  });
  const summary = computed<string>(() => translateWith('matchingLines', {
    count: search.get().hits.length
  }) + (search.get().skipped > 0 ? ' • ' + translateWith('skippedFiles', {
    count: search.get().skipped
  }) : ''));
  return (
    <View className={panelClass.get()}>
      <Ticker
        interval={50}
        running={search.get().pending || search.get().running}
        onTick={(_frame) => {
          pollContentSearch(workspace, settings, search, sender, replies);
        }}
      />
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
              next.generation += 1;
              next.pending = false;
              next.searched = false;
              next.hits = [];
              next.error = '';
              search.set(next);
            }}
            onSubmit={(_secondary) => {
              submitContentSearch(search);
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
                submitContentSearch(search);
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
                submitContentSearch(search);
              }}
            />
            <InputGroupButton
              icon="search"
              loading={search.get().running || search.get().pending}
              tooltip={message("Run content search")}
              accessibilityLabel={message("Run content search")}
              onClick={() => {
                submitContentSearch(search);
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
