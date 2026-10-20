import { View, Text, Node, signal } from 'std:ui';
import { Table, TableHeader, TableBody, TableFooter, TableRow, TableHead, TableCell, TableCaption } from '../components/ui/table';
import { Button } from '../components/ui/button-v3';

export function InvoiceTable(): Node {
  return (
    <Table id="invoice-table">
      <TableCaption>A list of your recent invoices.</TableCaption>
      <TableHeader><TableRow><TableHead scope="col" className="w-[100px]">Invoice</TableHead><TableHead scope="col">Status</TableHead><TableHead scope="col">Method</TableHead><TableHead scope="col" className="text-right">Amount</TableHead></TableRow></TableHeader>
      <TableBody>
        <TableRow><TableCell className="font-medium">INV001</TableCell><TableCell>Paid</TableCell><TableCell>Credit Card</TableCell><TableCell className="text-right">$250.00</TableCell></TableRow>
        <TableRow><TableCell className="font-medium">INV002</TableCell><TableCell>Pending</TableCell><TableCell>PayPal</TableCell><TableCell className="text-right">$150.00</TableCell></TableRow>
        <TableRow><TableCell className="font-medium">INV003</TableCell><TableCell>Unpaid</TableCell><TableCell>Bank Transfer</TableCell><TableCell className="text-right">$350.00</TableCell></TableRow>
      </TableBody>
      <TableFooter><TableRow><TableCell colSpan={3}>Total</TableCell><TableCell className="text-right">$750.00</TableCell></TableRow></TableFooter>
    </Table>
  );
}

export function TableStates(): Node {
  const selected = signal(false);
  const roomy = signal(false);
  return (
    <View id="table-states" className="gap-4 min-w-0">
      <Table id="state-table" className={roomy.get() ? 'text-base' : 'text-sm'}>
        <TableCaption>Selection belongs to application state.</TableCaption>
        <TableHeader><TableRow><TableHead scope="col">Project</TableHead><TableHead scope="col">Runtime</TableHead><TableHead scope="col">State</TableHead></TableRow></TableHeader>
        <TableBody>
          <TableRow id="selectable-table-row" data-state={selected.get() ? 'selected' : undefined}>
            <TableHead scope="row" className={roomy.get() ? 'h-16 px-4' : 'h-10 px-2'}>Documentation</TableHead><TableCell>Web + native</TableCell><TableCell>{selected.get() ? 'Selected' : 'Available'}</TableCell>
          </TableRow>
          <TableRow><TableHead scope="row">Invoice viewer</TableHead><TableCell>Native</TableCell><TableCell>Available</TableCell></TableRow>
        </TableBody>
      </Table>
      <View className="flex-row flex-wrap gap-2"><Button variant="outline" onPress={() => { selected.set(!selected.get()); }}>Toggle row selection</Button><Button variant="outline" onPress={() => { roomy.set(!roomy.get()); }}>Toggle table density</Button></View>
    </View>
  );
}

export function TableOverflow(): Node {
  return (
    <View id="table-overflow" className="w-full min-w-0 gap-3">
      <Text className="text-sm text-muted-foreground">Scroll the table to inspect every column.</Text>
      <Table id="wide-table" className="min-w-192">
        <TableCaption>Build artifacts across runtime targets.</TableCaption>
        <TableHeader><TableRow><TableHead scope="col">Artifact</TableHead><TableHead scope="col">Target</TableHead><TableHead scope="col">Architecture</TableHead><TableHead scope="col">Availability</TableHead></TableRow></TableHeader>
        <TableBody><TableRow><TableHead scope="row">Documentation application</TableHead><TableCell>Native GPU</TableCell><TableCell>Apple Silicon</TableCell><TableCell>Local build</TableCell></TableRow><TableRow><TableHead scope="row">Documentation application</TableHead><TableCell>WebAssembly</TableCell><TableCell>Browser</TableCell><TableCell>Local build</TableCell></TableRow></TableBody>
      </Table>
    </View>
  );
}
