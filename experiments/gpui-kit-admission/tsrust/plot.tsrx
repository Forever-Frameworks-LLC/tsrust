import { View, Text, Node, signal, run, Plot, PlotLine, PlotArea, PlotBars, SpeechButton } from 'std:ui';

function App(): Node {
  const heard = signal('');
  const status = signal('idle');
  return (
    <View className="flex-col size-full p-6 gap-4">
      <Plot labels={['Q1', 'Q2', 'Q3', 'Q4', 'Q5']} axis={true} grid={true} className="h-64">
        <PlotArea values={[2, 3, 2.5, 4, 3.5]} color="chart-3" />
        <PlotBars values={[4, 6, 5, 8, 7]} />
        <PlotBars values={[3, 5, 6, 6, 9]} color="chart-2" />
        <PlotLine values={[3, 5, 6, 7, 8]} curve="linear" dots={true} color="destructive" />
      </Plot>
      <View className="flex-row gap-3 items-center">
        <SpeechButton waveform={true} onPartial={(text) => { heard.set(clone(text)); status.set('listening'); }} onFinal={(text) => { heard.set(clone(text)); status.set('done'); }} onError={(message) => { status.set(clone(message)); }} />
        <Text>{status}: {heard}</Text>
      </View>
    </View>
  );
}

function main() {
  unwrap(run(App(), 'Plot and speech', {
    width: 900, height: 520
  }));
}
