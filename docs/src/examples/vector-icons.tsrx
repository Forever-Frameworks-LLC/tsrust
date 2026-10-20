import { View, Text, Button, Svg, Path, Node, signal } from 'std:ui';

function CheckIcon(): Node {
  return (<Svg className="size-4" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round"><Path d="M4 12L9 17L20 6"/></Svg>);
}

function ArrowIcon(): Node {
  return (<Svg className="size-4" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round"><Path d="M5 12H19M13 6L19 12L13 18"/></Svg>);
}

export function VectorIcons(): Node {
  const saved = signal(false);
  return (
    <View className="gap-4">
      <View className="flex-row flex-wrap gap-3 items-center">
        <Button id="vector-save" className="rounded bg-primary text-primary-foreground px-3 py-2 gap-2" onPress={() => { saved.set(!saved.get()); }}><CheckIcon/>Save</Button>
        <Button className="rounded border px-3 py-2 gap-2">Continue<ArrowIcon/></Button>
      </View>
      <Text id="vector-save-status">{saved.get() ? 'Saved' : 'Not saved'}</Text>
      <Text className="text-sm text-muted-foreground">Icons inherit text color and stay out of the accessibility name. The button owns its action.</Text>
    </View>
  );
}
