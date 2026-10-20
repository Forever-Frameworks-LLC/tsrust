// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ViewProps, Node } from 'std:ui';

export interface AspectRatioProps extends ViewProps {
  /** Required construction-time width divided by height. State changes do not update this prop. Must be finite, positive and within native f32 layout range. No default. Use 16.0 / 9.0 for a widescreen ratio. */
  ratio: number;
}

/** Reserves a preferred width/height ratio. Explicit dimensions and min/max constraints still apply. Caller classes override the ratio; children transfer once. Image loading and fitting are separate capabilities. */
export function AspectRatio(props: AspectRatioProps): Node {
  const slot = clone(props['data-slot']) ?? 'aspect-ratio';
  const classes = 'relative w-full min-w-0 shrink-0 aspect-[' + String(props.ratio) + '] ' + (clone(props.className) ?? '');
  return (<View {...props} data-slot={slot} className={classes} />);
}
