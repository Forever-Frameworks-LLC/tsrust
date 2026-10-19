// License: MIT. See ../../../attributions/shadcn-MIT.txt.
/*
Adapted from shadcn-ui/ui daaf156ee00ed91cc7e6bdd937c1b4973152d141, Base UI + Nova.
Portable text controls; specialized input types, DOM event methods and animated transitions remain unsupported.
*/
import { Input as PrimitiveInput, InputProps, Node } from 'std:ui';

/** Nova single-line text field. The project owns this source and its theme tokens. Other input types are not implemented yet. */
export function Input(props: InputProps): Node {
  const slot = clone(props['data-slot']) ?? 'input';
  const classes = 'w-full min-w-0 outline-none placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 h-8 py-1 border-input focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 disabled:bg-input/50 dark:disabled:bg-input/80 rounded-lg border bg-transparent px-2.5 text-base focus-visible:ring-3 aria-invalid:ring-3 dark:bg-input/30 md:text-sm aria-invalid:focus-visible:border-destructive dark:aria-invalid:focus-visible:border-destructive/50 aria-invalid:focus-visible:ring-destructive/20 dark:aria-invalid:focus-visible:ring-destructive/40 ' + (clone(props.className) ?? '');
  return (<PrimitiveInput {...props} data-slot={slot} className={classes}/>);
}
