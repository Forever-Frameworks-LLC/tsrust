import {
  View,
  Text,
  Input,
  Button,
  ScrollArea,
  Ticker,
  Node,
  signal,
  computed,
  run,
  quit
} from 'std:ui';
import { get } from 'std:env';
import { makeItems, filterItems } from './data.tsr';
import { Row } from './row.tsrx';

// BENCH_MODE selects an automated run: startup quits after the first tick,
// idle quits after 10 seconds, animate refilters the list every 16 ms for 600 ticks.
function App(mode: string): Node {
  const items = signal(makeItems(2000));
  const query = signal('');
  const frame = signal(0);
  // computed holds only scalars, so the count and the rows each filter the list.
  const count = computed<int>(() => filterItems(items.get(), query.get()).length);
  const interval: number = mode === 'idle' ? 10000 : 16;
  return (
    <View className="flex flex-col w-full h-full p-4 gap-3">
      <Text className="text-lg font-semibold">
        {'Showing ' + count.get().toString() + ' items, frame ' + frame.get().toString()}
      </Text>
      <View className="flex flex-row gap-2">
        <Input
          className="flex-1"
          placeholder="Search"
          value={query.get()}
          onChange={(event) => {
            query.set(clone(event.target.value));
          }}
        />
        <Button onPress={() => {
          const next = items.get();
          next.push('Item ' + next.length.toString());
          items.set(next);
        }}>Add item</Button>
      </View>
      <ScrollArea className="flex-1">
        {...filterItems(items.get(), query.get()).map((item: ReadOnly<string>) => Row(item))}
      </ScrollArea>
      <Ticker
        interval={interval}
        running={mode !== ''}
        onTick={(tick) => {
          if (mode !== 'animate' || tick >= 600) {
            quit();
            return;
          }

          frame.set(tick);
          query.set((tick % 10).toString());
        }}
      />
    </View>
  );
}

function main() {
  const mode = unwrap(get('BENCH_MODE')) ?? '';
  unwrap(run(App(mode), 'GPUI compare', {
    width: 900, height: 700
  }));
}
