import {
  View,
  Text,
  Button,
  Input,
  DataTable,
  DataTableColumn,
  signal,
  Node,
  run,
  setTheme
} from 'std:ui';

function App(): Node {
  const rows = signal('[["9007199254740993","first"],["9007199254740992","second"],["2","third"],["02","fourth"]]');
  const selected = signal(-1);
  return (
    <View className="size-full flex-col p-4 gap-3 bg-background text-foreground text-[14px]">
      <Text>Controlled: the handler deliberately retains descending.</Text>
      <View className="h-[180px]">
        <DataTable rows='[["controlled a"],["controlled z"]]' accessibilityLabel="Controlled table" columnMovable={false} onSort={(sort) => {}} className="size-full">
          <DataTableColumn key="0" width={420} sortable={true} sort="descending">Controlled</DataTableColumn>
        </DataTable>
      </View>
      <View className="flex-row gap-3">
        <Button onPress={() => { rows.set('[["9","first"],["3","second"],["3","third"],["1","fourth"]]'); }}>Replace rows</Button>
        <Button onPress={() => { rows.set('[["9007199254740993","first"],["9007199254740992","second"],["2","third"],["02","fourth"]]'); }}>Restore rows</Button>
        <Input aria-label="Selected source row" readOnly={true} value={selected.get().toString()} />
      </View>
      <Text>Uncontrolled: preserve sorting, widths and order when data refreshes.</Text>
      <View className="grow min-h-0">
        <DataTable
          rows={rows.get()}
          accessibilityLabel="Uncontrolled table"
          className="size-full"
          onSelect={(row) => {
            selected.set(row);
          }}
        >
          <DataTableColumn key="0" width={420} sortable={true} defaultSort="ascending">Uncontrolled</DataTableColumn>
          <DataTableColumn key="1" width={200} sortable={true}>Source identity</DataTableColumn>
        </DataTable>
      </View>
    </View>
  );
}

function main(): void {
  setTheme('dark');
  unwrap(run(App(), 'Table state acceptance', {
    width: 760, height: 640
  }));
}
