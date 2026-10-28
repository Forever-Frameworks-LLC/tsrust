import { View, Button, Node, signal, withTheme, run } from 'std:ui';
import { InvoiceTable, TableStates, TableOverflow } from '../examples/table-layouts';
import config from '../../tsrust.config.json';

function Page(): Node {
  const mode = signal('light');
  const content = (
    <View id="table-acceptance" className="h-screen overflow-auto p-4 gap-8 bg-background text-foreground">
      <Button id="toggle-table-theme" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Toggle table theme</Button>
      <InvoiceTable/>
      <TableStates/>
      <View className="w-64 min-w-0"><TableOverflow/></View>
    </View>
  );
  return withTheme(content, config.ui.theme, mode);
}

function main() {
  unwrap(run(Page(), 'Table acceptance', {
    width: 900, height: 780
  }));
}
