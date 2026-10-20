import { View, Text, Button, Svg, Path, Node, signal } from 'std:ui';

export function VectorChart(): Node {
  const high = signal(false);
  return (
    <View className="gap-4">
      <Svg id="vector-chart" viewBox="0 0 400 200" className="w-full h-48 bg-card" preserveAspectRatio="none" aria-label="Weekly successful builds" aria-describedby="vector-values" fill="none">
        <Path d="M20 20V180H380" stroke="border"/>
        <Path id="vector-area" d={high.get() ? 'M20 180L20 130L92 95L164 110L236 60L308 85L380 25L380 180Z' : 'M20 180L20 160L92 130L164 145L236 100L308 115L380 80L380 180Z'} fill="primary" className="opacity-50"/>
        <Path id="vector-series" d={high.get() ? 'M20 130L92 95L164 110L236 60L308 85L380 25' : 'M20 160L92 130L164 145L236 100L308 115L380 80'} stroke="primary" strokeWidth={2} strokeLinejoin="round"/>
      </Svg>
      <Text id="vector-values" className="text-sm">{high.get() ? 'Builds by day: 50, 85, 70, 120, 95, 155.' : 'Builds by day: 20, 50, 35, 80, 65, 100.'}</Text>
      <Button id="vector-change-series" className="rounded border px-3 py-2 self-start" onPress={() => { high.set(!high.get()); }}>Change series</Button>
    </View>
  );
}
