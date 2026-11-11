import {
  View, Text, Node, Button, Ticker, Plot, PlotLine, PlotArea, ProgressCircle, Progress, ShimmerText, Spinner,
  Carousel, CarouselContent, CarouselItem, CarouselPagination, Tag, Kbd, signal
} from 'std:ui';

/** A smooth 0..1 wave for column x, row y at time t. */
function wave(x: number, y: number, t: number): number {
  return (Math.sin(x * 0.55 + t * 0.09) + Math.cos(y * 0.7 - t * 0.06) + 2) / 4;
}

/** The palette step for a 0..1 level. */
function shade(level: number): string {
  if (level < 0.2) {
    return '100';
  } else if (level < 0.35) {
    return '200';
  } else if (level < 0.5) {
    return '300';
  } else if (level < 0.65) {
    return '400';
  } else if (level < 0.8) {
    return '500';
  }

  return '600';
}

/** One field cell: its shade follows the wave, so 160 cells recolor every frame. */
function cellClass(x: int, y: int, t: int): string {
  return `size-[14px] rounded-sm bg-orange-${shade(wave(Number(x), Number(y), Number(t)))}`;
}

/** An equalizer bar height in pixels. */
function barHeight(index: int, t: int): number {
  const i = Number(index);
  const time = Number(t);
  return Math.round(12 + Math.abs(Math.sin(i * 0.8 + time * 0.12) * Math.cos(i * 0.3 + time * 0.05)) * 96);
}

/** A point on a circle around (cx, cy), rounded to a pixel. */
function orbit(center: number, radius: number, angle: number, cosine: boolean): number {
  return Math.round(center + (cosine ? Math.cos(angle) : Math.sin(angle)) * radius);
}

/** 48 samples of a moving series. */
function series(phase: number, speed: number, t: int): number[] {
  const values: number[] = [];
  const time = Number(t);
  for (let index = 0; index < 48; index += 1) {
    const i = Number(index);
    values.push(50 + Math.sin(i * 0.26 + time * speed + phase) * 30 + Math.sin(i * 0.9 + time * speed * 2.3) * 8);
  }

  return values;
}

/** One row of the wave field: 20 cells whose shades follow the wave. */
function fieldRow(y: int, t: int): Node {
  const cells: Node[] = [];
  for (let x = 0; x < 20; x += 1) {
    cells.push(<View className={cellClass(x, y, t)} />);
  }

  return <View className="flex-row gap-1">{...cells}</View>;
}

/** Three moving series on one plot; rebuilt each frame from the frame count. */
function scope(t: int): Node {
  const labels: string[] = [];
  for (let index = 0; index < 48; index += 1) {
    labels.push(`${index}`);
  }

  return (
    <Plot labels={labels} axis={false} grid={true}>
      <PlotArea values={series(0, 0.08, t)} color="orange-500" />
      <PlotLine values={series(2.1, 0.05, t)} color="orange-300" width={2} />
      <PlotLine values={series(4.2, 0.11, t)} color="stone-400" curve="step" />
    </Plot>
  );
}

/** The live render showcase: every panel is redrawn from signals on a 60 Hz Ticker, all in native GPUI. */
export function ShowcasePage(): Node {
  const frame = signal(0);
  const slide = signal(0);
  const running = signal(true);
  const rows: int[] = [0, 1, 2, 3, 4, 5, 6, 7];
  const bars: int[] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];
  return (
    <View className="flex-col gap-8">
      <Ticker interval={16} running={running.get()} onTick={(tick) => { frame.set(frame.get() + 1); if (frame.get() % 180 === 0) { slide.set((slide.get() + 1) % 3); } }} />
      <View className="flex-row items-end gap-8">
        <View className="flex-col gap-3 grow">
          <Text className="text-4xl font-semibold leading-tight">Rendered natively, every frame</Text>
          <Text className="text-muted-foreground max-w-[560px]">Each panel below is redrawn from signals sixty times a second by GPUI. No web view, no canvas, just Rust.</Text>
        </View>
        <View className="flex-col items-end gap-2">
          <Text className="text-5xl font-semibold text-orange-500">{frame.get()}</Text>
          <Text variant="muted">frames rendered</Text>
          <Button variant="primary" icon={running.get() ? 'pause' : 'play'} onPress={() => { running.set(!running.get()); }}>{running.get() ? 'Pause' : 'Resume'}</Button>
        </View>
      </View>
      <View className="flex-row gap-4">
        <View className="flex-col gap-3 grow rounded-xl border border-border p-5">
          <View className="flex-row items-center gap-2">
            <Text className="font-semibold">Signal scope</Text>
            <View className="grow" />
            <Tag variant="warning">3 series</Tag>
            <Tag>48 points</Tag>
          </View>
          <View className="h-56">
            {...[scope(frame.get())]}
          </View>
        </View>
        <View className="flex-col gap-4 w-[280px] rounded-xl border border-border p-5">
          <Text className="font-semibold">Load</Text>
          <View className="flex-row justify-between">
            <ProgressCircle size="large" value={Math.round(wave(1, 2, Number(frame.get())) * 100)} color="orange-500" />
            <ProgressCircle size="large" value={Math.round(wave(5, 1, Number(frame.get())) * 100)} color="orange-300" />
            <ProgressCircle size="large" value={Math.round(wave(9, 4, Number(frame.get())) * 100)} color="stone-500" />
          </View>
          <Progress value={(Number(frame.get()) * 0.4) % 100} color="orange-500" />
          <View className="flex-row items-center gap-2">
            <Spinner />
            <ShimmerText>Compiling the next frame</ShimmerText>
          </View>
        </View>
      </View>
      <View className="flex-row gap-4">
        <View className="flex-col gap-3 rounded-xl border border-border p-5">
          <Text className="font-semibold">Wave field</Text>
          <Text variant="muted">160 views recolored per frame</Text>
          <View className="flex-col gap-1">
            {...rows.map((y: int) => fieldRow(y, frame.get()))}
          </View>
        </View>
        <View className="flex-col gap-3 grow rounded-xl border border-border p-5">
          <Text className="font-semibold">Spectrum</Text>
          <View className="flex-row items-end gap-1 h-[112px]">
            {...bars.map((index: int) => <View className={`w-[10px] rounded-sm bg-orange-${index % 3 === 0 ? '500' : index % 3 === 1 ? '400' : '300'} h-[${barHeight(index, frame.get())}px]`} />)}
          </View>
          <View className="relative h-[120px] rounded-lg bg-muted">
            <View className={`absolute size-[18px] rounded-full bg-orange-500 left-[${orbit(205, 170, Number(frame.get()) / 18, true)}px] top-[${orbit(51, 40, Number(frame.get()) / 18, false)}px]`} />
            <View className={`absolute size-[10px] rounded-full bg-stone-500 left-[${orbit(210, 100, Number(frame.get()) / 9 + 2, true)}px] top-[${orbit(55, 25, Number(frame.get()) / 9 + 2, false)}px]`} />
            <View className={`absolute size-[6px] rounded-full bg-orange-300 left-[${orbit(214, 50, 0 - Number(frame.get()) / 6, true)}px] top-[${orbit(57, 14, 0 - Number(frame.get()) / 6, false)}px]`} />
          </View>
        </View>
      </View>
      <View className="flex-row gap-4 items-center">
        <View className="w-[420px]">
          <Carousel loop={true} selected={slide.get()}>
            <CarouselContent>
              <CarouselItem><View className="flex-col h-36 justify-center gap-1 rounded-xl bg-orange-500 p-6"><Text className="text-2xl font-semibold text-white">GPU text</Text><Text className="text-white">Glyphs rasterized by GPUI</Text></View></CarouselItem>
              <CarouselItem><View className="flex-col h-36 justify-center gap-1 rounded-xl bg-stone-700 p-6"><Text className="text-2xl font-semibold text-white">Kit widgets</Text><Text className="text-white">Every component, native</Text></View></CarouselItem>
              <CarouselItem><View className="flex-col h-36 justify-center gap-1 rounded-xl bg-orange-300 p-6"><Text className="text-2xl font-semibold text-stone-900">Signals</Text><Text className="text-stone-900">Only what changed redraws</Text></View></CarouselItem>
            </CarouselContent>
            <CarouselPagination size="small" />
          </Carousel>
        </View>
        <View className="flex-col gap-2 grow">
          <Text className="font-semibold">Try it</Text>
          <View className="flex-row items-center gap-2"><Text variant="muted">Press</Text><Kbd>secondary-k</Kbd><Text variant="muted">and search for "chart", "loading" or "spreadsheet".</Text></View>
        </View>
      </View>
    </View>
  );
}
