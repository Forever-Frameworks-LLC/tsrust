import { View, Text, Node } from 'std:ui';

export function ParentPositioning(): Node {
  return (
    <View className="gap-4">
      <Text>Absolute placement, stretched insets, relative offsets and negative offsets.</Text>
      <View id="position-frame-0" className="w-64 max-w-full h-48 border-2 p-4"><View id="position-child-0" className="absolute top-2 right-2 w-8 h-8 bg-primary"/></View>
      <View id="position-frame-1" className="w-64 max-w-full h-48 border-2 p-4"><View id="position-child-1" className="absolute inset-x-4 top-1/2 h-8 bg-primary"/></View>
      <View id="position-frame-2" className="w-64 max-w-full h-48 border-2 p-4"><View id="position-child-2" className="relative top-2 left-1/4 w-8 h-8 shrink-0 bg-primary"/></View>
      <View id="position-frame-3" className="w-64 max-w-full h-48 border-2 p-4"><View id="position-child-3" className="absolute -left-2 -top-2 w-8 h-8 bg-primary"/></View>
    </View>
  );
}
