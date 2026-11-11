import { Recovery } from '../../lib/workspace/recovery.tsr';
import { publishWorkspace } from '../../lib/workspace/document-actions.tsr';
import { accessibleName, message, localizeError } from '../../lib/localization.tsr';
import { View, Text, Input, InputGroup, InputGroupAddon, InputGroupButton, Button, Node, Signal, signal, computed, controlInput, translateWith } from 'std:ui';
import { Workspace, openFile } from '../../lib/workspace/workspace.tsr';
import { SearchHit, searchContents } from '../../lib/explorer/search.tsr';

function ResultRow(hit: ReadOnly<SearchHit>, root: string, workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  const rowClass = computed<string>(() => 'w-full justify-start text-[' + font.get().toString() + 'px]');
  const path = clone(hit.path);
  const offset = hit.offset;
  return <Button variant="ghost" className={rowClass.get()} onPress={() => {
    const next = workspace.get();
    match (openFile(next, path)) {
      Ok(_) => { publishWorkspace(next, workspace, recovery); controlInput(clone(path), 'setCursor', { start: offset }); controlInput(clone(path), 'focus'); },
      Err(message) => { next.error = clone(message); publishWorkspace(next, workspace, recovery); }
    }
  }}><Text className="truncate">{hit.path.slice(root.length + 1) + ':' + hit.line.toString() + '  ' + hit.preview.trim()}</Text></Button>;
}

export function ContentSearch(workspace: Signal<Workspace>, recovery: Signal<Recovery>, font: Signal<int>): Node {
  const fontClass = computed<string>(() => 'text-[' + font.get().toString() + 'px]');
  const panelClass = computed<string>(() => 'flex-col size-full min-h-0 p-2 gap-2 text-[' + font.get().toString() + 'px]');
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
  return <View className={panelClass.get()}>
    <InputGroup size="small" className="w-full min-w-0">
    <Input value={query.get()} placeholder={message("Search file contents")} aria-label={accessibleName("Search file contents")} className={fontClass.get()} onChange={(event) => { query.set(clone(event.target.value)); }} />
    <InputGroupAddon align="inline-end">
      <InputGroupButton icon="case-sensitive" selected={sensitive.get()} tooltip={message("Match case")} accessibilityLabel={message("Match case")} onClick={() => { sensitive.set(!sensitive.get()); }} />
      <InputGroupButton icon="regex" selected={regex.get()} tooltip={message("Use regular expression")} accessibilityLabel={message("Use regular expression")} onClick={() => { regex.set(!regex.get()); }} />
      <InputGroupButton icon="search" tooltip={message("Run content search")} accessibilityLabel={message("Run content search")} onClick={() => {
        match (searchContents(workspace.get(), query.get(), regex.get(), sensitive.get())) {
          Ok(result) => { hits.set(result.hits); skipped.set(result.skipped); error.set(''); },
          Err(message) => { hits.set([]); error.set(message); }
        }
      }} />
    </InputGroupAddon>
    </InputGroup>
    <Text>{summary.get()}</Text><View hidden={error.get() === ''}><Text className="text-destructive">{localizeError(error.get())}</Text></View>
    <View className="flex-col grow min-h-0 overflow-y-auto">{...hits.get().map((hit: ReadOnly<SearchHit>) => ResultRow(hit, workspace.get().root, workspace, recovery, font))}</View>
  </View>;
}
