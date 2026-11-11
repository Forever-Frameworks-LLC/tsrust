import { View, Text, Input, Button, Node, Signal, signal, computed, controlInput } from 'std:ui';
import { Workspace, openFile } from './workspace.tsr';
import { SearchHit, searchContents } from './search.tsr';

function ResultRow(hit: ReadOnly<SearchHit>, root: string, workspace: Signal<Workspace>): Node {
  const path = clone(hit.path);
  const offset = hit.offset;
  return <Button variant="ghost" className="w-full justify-start" onPress={() => {
    const next = workspace.get();
    match (openFile(next, path)) {
      Ok(_) => { workspace.set(next); controlInput(clone(path), 'setCursor', { start: offset }); controlInput(clone(path), 'focus'); },
      Err(message) => { next.error = clone(message); workspace.set(next); }
    }
  }}><Text className="truncate">{hit.path.slice(root.length + 1) + ':' + hit.line.toString() + '  ' + hit.preview.trim()}</Text></Button>;
}

export function ContentSearch(workspace: Signal<Workspace>): Node {
  const query = signal('');
  const regex = signal(false);
  const sensitive = signal(false);
  const hits = signal<SearchHit[]>([]);
  const error = signal('');
  const skipped = signal(0);
  const summary = computed<string>(() => hits.get().length.toString() + ' matching lines • ' + skipped.get().toString() + ' files skipped');
  return <View className="flex-col size-full min-h-0 p-2 gap-2">
    <Input value={query.get()} placeholder="Search file contents" aria-label="Search file contents" className="w-full min-w-0" onChange={(event) => { query.set(clone(event.target.value)); }} />
    <View className="flex-row items-center gap-1">
      <Button variant="ghost" size="small" selected={sensitive.get()} tooltip="Match case" onPress={() => { sensitive.set(!sensitive.get()); }}>Aa</Button>
      <Button variant="ghost" size="small" selected={regex.get()} tooltip="Use regular expression" onPress={() => { regex.set(!regex.get()); }}>.*</Button>
      <Button icon="search" variant="ghost" size="small" tooltip="Run content search" aria-label="Run content search" onPress={() => {
        match (searchContents(workspace.get(), query.get(), regex.get(), sensitive.get())) {
          Ok(result) => { hits.set(result.hits); skipped.set(result.skipped); error.set(''); },
          Err(message) => { hits.set([]); error.set(message); }
        }
      }} />
    </View>
    <Text>{summary.get()}</Text><Text className="text-destructive">{error.get()}</Text>
    <View className="flex-col grow min-h-0 overflow-y-auto">{...hits.get().map((hit: ReadOnly<SearchHit>) => ResultRow(hit, workspace.get().root, workspace))}</View>
  </View>;
}
