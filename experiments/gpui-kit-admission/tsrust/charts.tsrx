import { View, Node, signal, run, Chart, CodeEditor, Markdown } from 'std:ui';

function App(): Node {
  const code = signal('fn main() {\n    let total: i32 = (1..=10).sum();\n    println!("total = {total}");\n}\n');
  return (
    <View className="flex-col size-full p-4 gap-4">
      <View className="flex-row gap-4 h-56">
        <Chart kind="radar" labels={['Speed', 'Memory', 'Startup', 'Size', 'Safety']} values={[90, 70, 95, 60, 100]} className="w-72 h-56" />
        <Chart kind="candlestick" labels={['Mon', 'Tue', 'Wed', 'Thu', 'Fri']} open={[10, 12, 11, 14, 13]} high={[13, 14, 15, 16, 15]} low={[9, 10, 10, 12, 11]} close={[12, 11, 14, 13, 15]} className="w-72 h-56" />
        <Chart kind="sankey" labels={['Revenue', 'Profit', 'Cost', 'Tax']} links={[['Revenue', 'Profit', '45'], ['Revenue', 'Cost', '55'], ['Profit', 'Tax', '10']]} className="w-80 h-56" />
      </View>
      <CodeEditor language="rust" value={code.get()} onChange={(text) => { code.set(clone(text)); }} className="h-40" />
      <Markdown content={'```rust\nlet x: u32 = 42;\n```'} />
    </View>
  );
}

function main() {
  unwrap(run(App(), 'Charts and highlighting', {
    width: 1100, height: 640
  }));
}
