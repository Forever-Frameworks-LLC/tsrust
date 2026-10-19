// License: MIT. See ../../../attributions/shadcn-MIT.txt.
/*
Adapted from shadcn-ui/ui daaf156ee00ed91cc7e6bdd937c1b4973152d141, Base UI + Nova.
Portable text controls; specialized input types, DOM event methods and animated transitions remain unsupported.
*/
import { Textarea as PrimitiveTextarea, TextareaProps, Node } from 'std:ui';

/** Nova multiline text field. Height follows content within the caller's min/max bounds. */
export function Textarea(props: TextareaProps): Node {
  const slot = clone(props['data-slot']) ?? 'textarea';
  const classes = 'flex field-sizing-content min-h-16 w-full outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 py-2 border-input focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 disabled:bg-input/50 dark:disabled:bg-input/80 rounded-lg border bg-transparent px-2.5 text-base focus-visible:ring-3 aria-invalid:ring-3 dark:bg-input/30 md:text-sm aria-invalid:focus-visible:border-destructive dark:aria-invalid:focus-visible:border-destructive/50 aria-invalid:focus-visible:ring-destructive/20 dark:aria-invalid:focus-visible:ring-destructive/40 ' + (clone(props.className) ?? '');
  return (<PrimitiveTextarea {...props} data-slot={slot} className={classes}/>);
}
