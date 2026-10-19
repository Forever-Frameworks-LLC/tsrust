import { View, Node, document, run, withTheme } from 'std:ui';
import { strictEqual } from 'std:testing';
import config from '../../tsrust.config.json';
import { CardLayout } from '../examples/card-layout';
import { CardProject } from '../examples/card-project';
import { InputStates } from '../examples/input-states';
import { MultilineDraft } from '../examples/multiline-draft';
import { FieldLabels } from '../examples/field-labels';
import { KeyboardShortcuts } from '../examples/keyboard-shortcuts';
import { SectionDividers } from '../examples/section-dividers';

function Gallery(): Node {
  return (
    <View className="h-screen overflow-auto p-6 gap-6" role="region" aria-label="Component gallery" tabIndex={0}>
      <CardLayout /><CardProject /><InputStates /><MultilineDraft /><FieldLabels /><KeyboardShortcuts /><SectionDividers />
    </View>
  );
}

function main() {
  const html = document(Gallery(), 'Gallery');
  for (const expected of ['Team workspace', 'Compact card', 'Project settings', 'input-states-name', 'aria-invalid="true"', 'multiline-message', 'field-labels-explicit', 'kbd-group', 'aria-orientation="vertical"']) {
    strictEqual(html.includes(expected), true);
  }

  console.log('component-gallery assertions passed');
  unwrap(run(withTheme(Gallery(), config.ui.theme), 'Tsrust component gallery', {
    width: 1100, height: 780, minWidth: 480, minHeight: 360, titlebar: 'custom'
  }));
}
