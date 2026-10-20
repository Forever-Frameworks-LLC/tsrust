import { View, Text, document } from 'std:ui';
import { strictEqual, throws } from 'std:testing';
import { AspectRatio } from '../components/ui/aspect-ratio';

function main() {
  const html = document(<AspectRatio ratio={1.5} id="ratio-test"><Text>Owned content</Text></AspectRatio>, 'Aspect ratio');
  strictEqual(html.includes('aspect-ratio:1.5'), true);
  strictEqual(html.includes('position:relative'), true);
  strictEqual(html.includes('min-w-0'), true);
  strictEqual(html.includes('shrink-0'), true);
  strictEqual(html.includes('data-slot="aspect-ratio"'), true);
  strictEqual(html.includes('id="ratio-test"'), true);
  strictEqual(html.includes('Owned content'), true);
  const override = document(<AspectRatio ratio={1.5} className="aspect-square" />, 'Override');
  strictEqual(override.includes('aspect-square'), true);
  for (const ratio of [0.0, -1.0, Number.NaN, Number.POSITIVE_INFINITY, 1e100, 1e-100]) {
    throws(() => {
      const invalid = <AspectRatio ratio={ratio} />;
    });
  }

  console.log('Aspect ratio construction passed');
}
