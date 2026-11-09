import { View, Text, Node, Markdown } from 'std:ui';

/** The landing page: what the gallery is and how to use a component. */
export function Introduction(): Node {
  return (
    <View className="flex-col gap-4">
      <Text className="text-3xl font-semibold">GPUI Kit for Tsrust</Text>
      <Text className="text-muted-foreground">Every GPUI Kit component, written in .tsrx. Each page follows the GPUI Kit documentation: every example runs live above the code that produces it.</Text>
      <Markdown content={'## Use a component\n\nImport it from `std:ui`:\n\n```tsx\nimport { Button } from \'std:ui\';\n\nfunction Save(): Node {\n  return <Button variant="primary" icon="check">Save</Button>;\n}\n```\n\nOr copy an editable wrapper into your project with `tsrust ui add Button`.\n\n## Run this gallery\n\n```bash\n./examples/gpui-starter/dev.sh\n```\n\nSaving a file rebuilds and restarts the window.'} />
    </View>
  );
}
