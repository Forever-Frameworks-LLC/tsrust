import { View, Text, Svg, Path, Node } from 'std:ui';

function Case(props: {
  id: string;
  policy: string;
  label: string;
}): Node {
  return (
    <View className="gap-2 items-center">
      <Text className="text-xs text-muted-foreground">{clone(props.label)}</Text>
      <Svg id={props.id} width={96} height={96} viewBox="10 20 40 20" preserveAspectRatio={props.policy} aria-label={props.label} fill="primary" className="bg-muted">
        <Path d="M10 20H50V40H10Z" className="opacity-50"/>
        <Path d="M10 20L50 40M50 20L10 40" fill="none" stroke="currentColor"/>
      </Svg>
    </View>
  );
}

export function VectorViewports(): Node {
  return (
    <View className="flex-row flex-wrap gap-4">
      <Case id="vector-meet" policy="xMidYMid meet" label="Centered meet"/>
      <Case id="vector-slice" policy="xMidYMid slice" label="Centered slice"/>
      <Case id="vector-none" policy="none" label="Stretch"/>
      <Case id="vector-corner" policy="xMaxYMax meet" label="Bottom right"/>
    </View>
  );
}
