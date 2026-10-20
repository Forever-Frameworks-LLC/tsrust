// License: MIT. See ../../../attributions/shadcn-MIT.txt.
import { View, ViewProps, Node, ReactiveValue, useProp } from 'std:ui';

export interface TableProps extends ViewProps<ReactiveValue<string>, ReactiveValue<string>> {}

export interface TableHeaderProps extends TableProps {}

export interface TableBodyProps extends TableProps {}

export interface TableFooterProps extends TableProps {}

export interface TableHeadProps extends TableProps {}

export interface TableRowProps extends TableProps {}

export interface TableCellProps extends TableProps {}

export interface TableCaptionProps extends TableProps {}

/** A shared-column table in a local horizontal overflow container. */
export function Table(props: TableProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table';
  props.className = undefined;
  return (
    <View data-slot="table-container" className="relative w-full min-w-0 overflow-auto">
      <View {...props} as="table" data-slot={slot} className={'table w-full caption-bottom text-sm ' + classes.get()}/>
    </View>
  );
}

/** The table's header slot; caller classes override its portable defaults. */
export function TableHeader(props: TableHeaderProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table-header';
  props.className = undefined;
  return (<View {...props} as="thead" data-slot={slot} className={'table-header-group [&_tr]:border-b ' + classes.get()}/>);
}

/** The table's body slot; caller classes override its portable defaults. */
export function TableBody(props: TableBodyProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table-body';
  props.className = undefined;
  return (<View {...props} as="tbody" data-slot={slot} className={'table-row-group [&_tr:last-child]:border-0 ' + classes.get()}/>);
}

/** The table's footer slot; caller classes override its portable defaults. */
export function TableFooter(props: TableFooterProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table-footer';
  props.className = undefined;
  return (<View {...props} as="tfoot" data-slot={slot} className={'table-footer-group bg-muted/50 border-t font-medium [&>tr:last-child]:border-b-0 ' + classes.get()}/>);
}

/** The table's head slot; caller classes override its portable defaults. */
export function TableHead(props: TableHeadProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table-head';
  props.className = undefined;
  return (<View {...props} as="th" data-slot={slot} className={'table-cell text-foreground h-10 px-2 text-left align-middle font-medium whitespace-nowrap ' + classes.get()}/>);
}

/** The table's row slot; caller classes override its portable defaults. */
export function TableRow(props: TableRowProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table-row';
  props.className = undefined;
  return (<View {...props} as="tr" data-slot={slot} className={'table-row hover:bg-muted/50 data-[state=selected]:bg-muted border-b has-aria-expanded:bg-muted/50 ' + classes.get()}/>);
}

/** The table's cell slot; caller classes override its portable defaults. */
export function TableCell(props: TableCellProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table-cell';
  props.className = undefined;
  return (<View {...props} as="td" data-slot={slot} className={'table-cell p-2 align-middle whitespace-nowrap ' + classes.get()}/>);
}

/** The table's caption slot; caller classes override its portable defaults. */
export function TableCaption(props: TableCaptionProps): Node {
  const classes = useProp<string>(props.className ?? '');
  const slot = clone(props['data-slot']) ?? 'table-caption';
  props.className = undefined;
  return (<View {...props} as="caption" data-slot={slot} className={'table-caption text-center text-muted-foreground mt-4 text-sm ' + classes.get()}/>);
}
