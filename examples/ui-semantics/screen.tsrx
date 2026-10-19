import { View, Text, Button, Node } from 'std:ui';

/** Semantic primitives are shared by editable local components and both renderers. */
export function Screen(): Node {
  return (
    <View as="section" role="region" aria-label="Account" data-slot="account" className="p-4 gap-4">
      <Text as="h2" id="profile-title" className="text-xl font-bold">Profile</Text>
      <Text as="p">Unicode: café · العربية · 🦀</Text>
      <Text role="alert" aria-label="Error summary">Please check your details.</Text>
      <Text role="status">Saved</Text>
      <View role="separator" aria-orientation="horizontal" />
      <Text as="kbd" aria-hidden>⌘ K</Text>
      <Button disabled aria-label="Save profile">Save</Button>
    </View>
  );
}
