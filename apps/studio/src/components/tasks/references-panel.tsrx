import { Sender } from 'std:futures';
import { LanguageMessage } from '../../services/lsp/session.tsr';
import { toInt } from 'std:numeric';
import {
  View,
  Shortcut,
  Ticker,
  Button,
  translate,
  translateWith,
  Text,
  Icon,
  List,
  ListItem,
  ListEmpty,
  Node,
  Signal,
  computed,
  t
} from 'std:ui';

import { SearchActionContext } from '../../lib/explorer/search-actions.tsr';
import { clearLanguageReferences, requestFileReferences, pollFileReferences } from '../../services/lsp/references.tsr';
import { documentUri } from '../../lib/language/diagnostics.tsr';
import { message, accessibleName } from '../../lib/localization.tsr';
import { openLanguageLocation } from '../../lib/language/navigation.tsr';
import { LanguageReferences, ReferenceLocation, referenceLabel } from '../../lib/language/references.tsr';
import { LanguageState } from '../../services/lsp/controller.tsr';
import { Recovery } from '../../lib/workspace/recovery.tsr';
import { Workspace } from '../../lib/workspace/workspace.tsr';

function ReferenceRow(root: ReadOnly<string>, location: ReadOnly<ReferenceLocation>, index: int, font: Signal<int>): Node {
  const label = referenceLabel(root, location);
  const rowClass = computed<string>(() => 'py-1 border-b border-border rounded-none text-[' + font.get().toString() + 'px]');
  return (
    <ListItem value={index.toString()} accessibilityLabel={clone(label)} className={rowClass.get()}>
      <Icon name="file-code" size="sm" className="shrink-0 text-muted-foreground" />
      <Text className="grow min-w-0 truncate font-mono">
        {label}
      </Text>
    </ListItem>
  );
}

function revealReference(workspace: Signal<Workspace>, recovery: Signal<Recovery>, references: Signal<LanguageReferences>, value: ReadOnly<string>): void {
  const index = unwrap(toInt(Number(value)));
  const result = references.get();

  if (result.root !== workspace.get().root || index < 0 || index >= result.locations.length) return;
  const root = clone(result.root);
  const context: SearchActionContext = {
    workspace, recovery, allowNavigation: true, allowedRoot: clone(root), focusEditor: true
  };
  const location = result.locations[index];

  match (openLanguageLocation(context, {
    root, uri: clone(location.uri), selection: clone(location.range)
  })) {
    Ok(_) => {},
    Err(error) => {
      const next = context.workspace.get();
      next.error = clone(error);
      context.workspace.set(next);
    }
  }
}

function clearReferences(workspace: Signal<Workspace>, recovery: Signal<Recovery>, references: Signal<LanguageReferences>): void {
  const root = clone(workspace.get().root);
  unwrap(clearLanguageReferences({
    workspace, recovery, references, allowedRoot: clone(root), allowNavigation: true, focusEditor: false
  }, root));
}

function revealReferenceSource(workspace: Signal<Workspace>, recovery: Signal<Recovery>, references: Signal<LanguageReferences>): void {
  const result = references.get();
  const query = result.query;

  if (query === undefined) return;
  const context: SearchActionContext = {
    workspace, recovery, allowNavigation: true, allowedRoot: clone(result.root), focusEditor: true
  };

  match (openLanguageLocation(context, {
    root: clone(result.root), uri: documentUri(query.path), selection: {
      start: clone(query.position), end: clone(query.position)
    }
  })) {
    Ok(_) => {},
    Err(error) => {
      const next = context.workspace.get();
      next.error = clone(error);
      context.workspace.set(next);
    }
  }
}

export function ReferencesPanel(workspace: Signal<Workspace>, recovery: Signal<Recovery>, references: Signal<LanguageReferences>, language: Signal<LanguageState>, font: Signal<int>): Node {
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 text-[' + font.get().toString() + 'px]');
  const sourceClass = computed<string>(() => 'w-full min-w-0 justify-start truncate font-mono text-[' + font.get().toString() + 'px]');
  const root = computed<string>(() => clone(references.get().root));
  const count = computed<int>(() => references.get().locations.length);
  const error = computed<string | undefined>(() => clone(references.get().error));
  const hasQuery = computed<boolean>(() => references.get().query !== undefined);
  const title = computed<string>(() => {
    const current = references.get();
    const query = current.query;

    if (query === undefined) return translate('No reference query');

    if (query.kind === 'file') return translate('File imports');
    return query.symbol === '' ? translate('Symbol references') : translateWith('References to {symbol}', {
      symbol: clone(query.symbol)
    });
  });
  const source = computed<string>(() => {
    const current = references.get();
    const query = current.query;

    if (query === undefined) return '';
    return referenceLabel(current.root, {
      uri: documentUri(query.path), range: {
        start: clone(query.position), end: clone(query.position)
      }
    });
  });
  const busy = computed<boolean>(() => references.get().pending === true || language.get().queries.some((query) => query.nativeRequest !== undefined && query.method === 'textDocument/references'));
  const incomplete = computed<boolean>(() => (references.get().issues?.length ?? 0) > 0);
  const scope = computed<string>(() => {
    const result = references.get();

    if (result.query?.kind !== 'file') return '';
    return translateWith('Direct Tsrust imports • {count} files scanned', {
      count: (result.scanned ?? 0).toString()
    });
  });
  return (
    <View className={panelClass.get()}>
      <View className="flex-row items-center gap-2 px-3 py-1 shrink-0 border-b border-border">
        <Text className="min-w-0 truncate">{title.get()} • {count.get().toString()}</Text>
        <View hidden={!hasQuery.get()} className="grow min-w-0">
          <Button
            variant="ghost"
            size="xsmall"
            className={sourceClass.get()}
            tooltip={source.get()}
            onPress={() => {
              revealReferenceSource(workspace, recovery, references);
            }}
          >
            {source.get()}
          </Button>
        </View>
        <Button
          icon="list-x"
          className="ml-auto shrink-0"
          variant="ghost"
          size="xsmall"
          tooltip={message('Clear references')}
          aria-label={accessibleName('Clear references')}
          disabled={!hasQuery.get() && count.get() === 0 && error.get() === undefined}
          onPress={() => {
            clearReferences(workspace, recovery, references);
          }}
        />
      </View>
      <View hidden={scope.get() === '' || busy.get()} className="px-3 py-1">
        <Text className="text-muted-foreground">
          {scope.get()}
        </Text>
      </View>
      <View hidden={!incomplete.get() || busy.get()} className="px-3 py-1">
        <Text className="text-muted-foreground">
          {t('Some imports could not be resolved. Results may be incomplete.')}
        </Text>
      </View>
      <View hidden={!busy.get()} className="p-3">
        <Text className="text-muted-foreground">
          {t('Finding references…')}
        </Text>
      </View>
      <View hidden={busy.get() || error.get() === undefined} className="p-3">
        <Text className="text-destructive">
          {t(error.get() ?? '')}
        </Text>
      </View>
      <View hidden={busy.get() || error.get() !== undefined || count.get() !== 0} className="p-3">
        <Text className="text-muted-foreground">
          {hasQuery.get() ? t('No references found.') : t('Right-click a symbol and choose Find References (Shift+F12).')}
        </Text>
      </View>
      <View hidden={busy.get() || error.get() !== undefined} className="flex-col grow min-h-0">
        <List
          className="grow min-h-0"
          onSelect={(value) => {
            revealReference(workspace, recovery, references, value);
          }}
          onConfirm={(value) => {
            revealReference(workspace, recovery, references, value);
          }}
        >
          {...references.get().locations.map((location: ReadOnly<ReferenceLocation>, index: int) => ReferenceRow(root.get(), location, index, font))}
          <ListEmpty>
            <View />
          </ListEmpty>
        </List>
      </View>
    </View>
  );
}

// Window commands and result delivery must remain mounted when the panel is closed.
export function ReferenceControls(
  workspace: Signal<Workspace>,
  recovery: Signal<Recovery>,
  references: Signal<LanguageReferences>,
  language: Signal<LanguageState>,
  panel: Signal<string>,
  messages: Sender<LanguageMessage>
): Node {
  const pollMessages = messages.share();
  const needsPoll = computed<boolean>(() => references.get().pending === true || language.get().queries.some((query) => query.id.startsWith('ui:file-references:')) || language.get().readResults.some((result) => result.query.id.startsWith('ui:file-references:')));
  return (
    <View>
      <Shortcut
        command="find-file-references"
        keys="secondary-alt-shift-f12"
        onTrigger={() => {
          const root = clone(workspace.get().root);
          const path = clone(workspace.get().active);
          const context: SearchActionContext = {
            workspace, recovery, references, panel, allowedRoot: root, allowNavigation: true, allowRead: true, focusEditor: false
          };

          match (requestFileReferences(context, language, messages, path)) {
            Ok(_) => {},
            Err(error) => {
              const next = context.workspace.get();
              next.error = clone(error);
              context.workspace.set(next);
            }
          }
        }}
      />
      <Ticker
        interval={150}
        onTick={(_frame) => {
          if (needsPoll.get()) pollFileReferences(workspace, references, language, pollMessages);
        }}
      />
    </View>
  );
}
