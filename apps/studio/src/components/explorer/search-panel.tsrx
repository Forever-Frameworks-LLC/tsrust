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
  Pagination,
  Node,
  Signal,
  signal,
  computed,
  translate,
  translateWith
} from 'std:ui';

import { SearchActionContext, fileSearchLabel, openSearchResult } from '../../lib/explorer/search-actions.tsr';
import { SearchHit, searchSnapshot, searchPreview, searchPage, searchPageCount } from '../../lib/explorer/search.tsr';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { StudioConfig } from '../../lib/settings/settings.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';

import { ContentSearchRequest, ContentSearchReply } from '../../services/content-search.tsr';

function ResultRow(
  hit: ReadOnly<SearchHit>,
  root: string,
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  font: Signal<int>,
  selected: Signal<string>
): Node {
  const key = clone(hit.path) + ':' + hit.offset.toString();
  const rowKey = clone(key);
  const rowClass = computed<string>(() => 'flex-col items-stretch justify-center gap-1 w-full min-w-0 shrink-0 px-2 py-2 rounded-none hover:bg-accent min-h-[' + (font.get() * 3 + 16).toString() + 'px] text-[' + font.get().toString() + 'px]' + (selected.get() === rowKey ? ' bg-accent text-accent-foreground' : ''));
  const parts = searchPreview(hit);
  const location = fileSearchLabel(root, hit.path) + ':' + hit.line.toString();
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
            Ok(_) => {
              selected.set(clone(key));
            },
            Err(message) => {
              const next = workspace.get();
              next.error = clone(message);
              publishWorkspace(next, workspace, recovery);
            }
          }
        }}
      >
        <Text className="w-full truncate text-muted-foreground">
          {fileSearchLabel(root, hit.path)}
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
  count: int;
  page: int;
  error: string;
  skipped: int;
  searched: boolean;
  generation: int;
  pending: boolean;
  running: boolean;
  active: int;
  cancelSent: boolean;
}

function submitContentSearch(search: Signal<ContentSearchState>, hits: Signal<SearchHit[]>, run: boolean = true): void {
  const next = search.get();
  next.generation += 1;
  next.pending = run && next.query !== '';
  next.searched = false;

  if (next.count > 0) hits.set([]);
  next.count = 0;
  next.page = 1;
  next.error = '';
  search.set(next);
}

function pollContentSearch(
  workspace: Signal<Workspace>,
  settings: Signal<StudioConfig>,
  search: Signal<ContentSearchState>,
  hits: Signal<SearchHit[]>,
  pageHits: Signal<SearchHit[]>,
  requests: ReadOnly<Sender<ContentSearchRequest>>,
  replies: Mutable<Channel<ContentSearchReply>>,
  cancellations: ReadOnly<Sender<int>>
): void {
  const next = search.get();

  if (next.running) {
    if (next.active !== next.generation && !next.cancelSent) {
      match (cancellations.trySend(next.active)) {
        Ok(_) => {
          next.cancelSent = true;
          search.set(next);
        },
        Err(_) => {}
      }
    }

    match (replies.tryReceive()) {
      Ok(reply) => {
        next.running = false;

        if (next.generation === reply.generation) {
          next.count = reply.result.hits.length;
          pageHits.set(searchPage(reply.result.hits, 1));
          hits.set(reply.result.hits);
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
      generation: next.generation, workspace: searchSnapshot(workspace.get()), query: clone(next.query), regex: next.regex, sensitive: next.sensitive, includeIgnored: settings.get().searchIncludeIgnored
    })) {
      Ok(_) => {
        next.running = true;
        next.active = next.generation;
        next.cancelSent = false;
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
  cancellations: Sender<int>,
  replies: Channel<ContentSearchReply>
): Node {
  const selected = signal('');
  const sender = requests.sender();
  const fontClass = computed<string>(() => 'px-2 text-[' + font.get().toString() + 'px]');
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 text-[' + font.get().toString() + 'px]');
  const search = signal<ContentSearchState>({
    query: '', regex: false, sensitive: false, count: 0, page: 1, error: '', skipped: 0, searched: false, generation: 0, pending: false, running: false, active: 0, cancelSent: false
  });
  const hits = signal<SearchHit[]>([]);
  const pageHits = signal<SearchHit[]>([]);
  const runLabel = computed<string>(() => search.get().running || search.get().pending ? translate("Cancel search") : translate("Run content search"));
  const pages = computed<int>(() => searchPageCount(search.get().count));
  const summary = computed<string>(() => translateWith('matchingLines', {
    count: search.get().count
  }) + (search.get().skipped > 0 ? ' • ' + translateWith('skippedFiles', {
    count: search.get().skipped
  }) : ''));
  return (
    <View className={panelClass.get()}>
      <Ticker
        interval={50}
        running={search.get().pending || search.get().running}
        onTick={(_frame) => {
          pollContentSearch(
            workspace,
            settings,
            search,
            hits.handle(),
            pageHits.handle(),
            sender,
            replies,
            cancellations
          );
        }}
      />
      <View className="shrink-0 h-8 min-w-0">
        <InputGroup size="small" className="w-full h-full min-w-0 rounded-none border border-transparent">
          <Input
            id="content-search"
            value={search.get().query}
            placeholder={message("Search file contents")}
            aria-label={accessibleName("Search file contents")}
            className={fontClass.get()}
            onChange={(event) => {
              const next = search.get();
              selected.set('');
              next.query = clone(event.target.value);
              search.set(next);
              submitContentSearch(search, hits.handle(), false);
            }}
            onSubmit={(_secondary) => {
              submitContentSearch(search, hits.handle());
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
                submitContentSearch(search, hits.handle());
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
                submitContentSearch(search, hits.handle());
              }}
            />
            <InputGroupButton
              icon={search.get().running || search.get().pending ? 'square' : 'search'}
              tooltip={runLabel.get()}
              accessibilityLabel={runLabel.get()}
              onClick={() => {
                submitContentSearch(search, hits.handle(), !(search.get().running || search.get().pending));
              }}
            />
          </InputGroupAddon>
        </InputGroup>
      </View>
      <View hidden={!search.get().searched || search.get().error !== ''} className="shrink-0 px-2">
        <Text>
          {summary.get()}
        </Text>
      </View>
      <View hidden={search.get().error === ''} className="shrink-0 px-2">
        <Text className="text-destructive">
          {localizeError(search.get().error)}
        </Text>
      </View>
      <View hidden={search.get().count === 0} className="flex-col grow min-h-0 overflow-y-auto">
        {...pageHits.get().map((hit: ReadOnly<SearchHit>) => ResultRow(
          hit,
          workspace.get().root,
          workspace,
          recovery,
          font,
          selected.handle()
        ))}
      </View>
      <View hidden={pages.get() <= 1} className="flex-row justify-center shrink-0 w-full min-w-0 overflow-x-auto">
        <Pagination
          page={search.get().page}
          total={pages.get() > 0 ? pages.get() : 1}
          size="xsmall"
          visiblePages={5}
          onChange={(page) => {
            const next = search.get();

            if (page >= 1 && page <= searchPageCount(next.count)) {
              pageHits.set(searchPage(hits.get(), page));
              next.page = page;
              search.set(next);
            }
          }}
        />
      </View>
    </View>
  );
}
