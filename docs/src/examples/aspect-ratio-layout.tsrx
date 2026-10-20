import { View, Text, Node } from 'std:ui';
import { AspectRatio } from '../components/ui/aspect-ratio';

export function AspectRatioLayout(): Node {
  return (
    <View className="grid grid-cols-1 md:grid-cols-2 items-start gap-4 p-1">
      <AspectRatio id="aspect-square-example" ratio={16.0 / 9.0} className="aspect-square w-full bg-muted rounded-lg items-center justify-center"><Text>Square override</Text></AspectRatio>
      <AspectRatio id="aspect-video-example" ratio={16.0 / 9.0} className="w-full bg-accent rounded-lg items-center justify-center"><Text>Widescreen</Text></AspectRatio>
      <AspectRatio id="aspect-portrait-example" ratio={3.0 / 4.0} className="w-full bg-muted rounded-lg items-center justify-center"><Text>Portrait</Text></AspectRatio>
      <AspectRatio id="aspect-fractional-width-example" ratio={1.5} className="w-full bg-accent rounded-lg items-center justify-center"><Text>Decimal ratio</Text></AspectRatio>
      <AspectRatio id="aspect-explicit-height-example" ratio={16.0 / 9.0} className="w-full h-32 bg-muted rounded-lg items-center justify-center"><Text>Explicit height</Text></AspectRatio>
      <AspectRatio id="aspect-minimum-height-example" ratio={16.0 / 9.0} className="w-full min-h-64 bg-accent rounded-lg items-center justify-center"><Text>Minimum height</Text></AspectRatio>
      <AspectRatio id="aspect-maximum-height-example" ratio={1.0} className="w-full max-h-48 bg-muted rounded-lg items-center justify-center"><Text>Maximum height</Text></AspectRatio>
    </View>
  );
}
