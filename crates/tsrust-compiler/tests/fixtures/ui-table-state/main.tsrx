import {
  View,
  Text,
  Link,
  Button,
  Input,
  DataTable,
  DataTableColumn,
  Select,
  SelectItem,
  InputGroup,
  InputGroupAddon,
  signal,
  Node,
  run,
  setTheme
} from 'std:ui';

function App(): Node {
  const rows = signal('[["9007199254740993","first"],["9007199254740992","second"],["2","third"],["02","fourth"]]');
  const selected = signal(-1);
  const choice = signal('first');
  return (
    <View className="size-full flex-col p-4 gap-3 bg-background text-foreground text-[14px]">
      <Text>Controlled: the handler deliberately retains descending.</Text>
      <Link href="https://example.com" className="text-[20px]">Reference link</Link>
      <View className="h-[180px]">
        <DataTable rows='[["controlled a"],["controlled z"]]' accessibilityLabel="Controlled table" columnMovable={false} onSort={(sort) => {}} className="size-full">
          <DataTableColumn key="0" width={420} sortable={true} sort="descending">Controlled</DataTableColumn>
        </DataTable>
      </View>
      <View className="flex-row gap-3 h-[40px]">
        <Select
          accessibilityLabel="Plain selection"
          value={choice.get()}
          className="w-[180px]"
          onChange={(value) => {
            choice.set(value);
          }}
        >
          <SelectItem value="first">First option</SelectItem>
          <SelectItem value="second">Second option</SelectItem>
          <SelectItem value="blocked" disabled={true}>Unavailable option</SelectItem>
        </Select>
        <InputGroup className="w-[360px]" accessibilityLabel="Grouped selection">
          <Input placeholder="Search" />
          <InputGroupAddon align="inline-end">
            <Select
              accessibilityLabel="Addon selection"
              appearance={false}
              value={choice.get()}
              className="w-[180px]"
              onChange={(value) => {
                choice.set(value);
              }}
            >
              <SelectItem value="first">First option</SelectItem>
              <SelectItem value="second">Second option</SelectItem>
              <SelectItem value="blocked" disabled={true}>Unavailable option</SelectItem>
            </Select>
          </InputGroupAddon>
        </InputGroup>
      </View>
      <View className="flex-row gap-3">
        <Button className="shrink-0 text-[20px]" variant="primary" onPress={() => { rows.set('[["9","first"],["3","second"],["3","third"],["1","fourth"]]'); }}>Replace rows</Button>
        <Button className="shrink-0 text-[20px]" variant="ghost" selected={true} onPress={() => { rows.set('[["9007199254740993","first"],["9007199254740992","second"],["2","third"],["02","fourth"]]'); }}>Restore rows</Button>
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
          <DataTableColumn key="amount" width={420} sortable={true} defaultSort="ascending">Uncontrolled</DataTableColumn>
          <DataTableColumn key="identity" width={200} sortable={true}>Source identity</DataTableColumn>
        </DataTable>
      </View>
    </View>
  );
}

function main(): void {
  setTheme('dark');
  unwrap(run(App(), 'Table state acceptance', {
    width: 760, height: 760
  }));
}
