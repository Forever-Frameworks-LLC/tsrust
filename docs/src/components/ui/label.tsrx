// License: MIT. See ../../../attributions/shadcn-MIT.txt.
/*
Adapted from shadcn-ui/ui daaf156ee00ed91cc7e6bdd937c1b4973152d141, Base UI + Nova.
Portable text controls; specialized input types, DOM event methods and animated transitions remain unsupported.
*/
import { View, ViewProps, Node } from 'std:ui';

/** Visible label. Set htmlFor to the control ID, or wrap exactly one control. Disabled state belongs to the control. */
export function Label(props: ViewProps): Node {
  const slot = clone(props['data-slot']) ?? 'label';
  const classes = 'flex flex-row items-center select-none text-sm font-medium ' + (clone(props.className) ?? '');
  return (<View {...props} as="label" data-slot={slot} className={classes}/>);
}
