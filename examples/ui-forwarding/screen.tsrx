import { View, ViewProps, Text, Node, withTheme } from 'std:ui';
import config from './tsrust.config.json';

/** Local props extend the shared interface; JSX moves declared base fields into View. */
interface PanelProps extends ViewProps {
  size?: 'sm' | 'default';
}

function Panel(props: PanelProps): Node {
  const size = String(props.size ?? 'default');
  const classes = 'group/panel group/tile p-4 gap-4 data-[size=sm]:p-3 has-data-[slot=footer]:pb-0 ' + (clone(props.className) ?? '');
  return (<View {...props} data-size={size} className={classes} />);
}

export function Screen(): Node {
  return withTheme((
    <View>
      <Panel size="sm">
        <View data-slot="body" className="p-4 group-data-[size=sm]/panel:px-3 min-h-0 group-data-[size=sm]/tile:group-data-[size=sm]/panel:min-h-8"><Text>Small panel</Text></View>
        <View data-slot="footer" className="border [.border]:pt-6"><Text>Footer</Text></View>
      </Panel>
      <Panel><View data-slot="body" className="p-4 group-data-[size=sm]/panel:px-3 min-h-0 group-data-[size=sm]/tile:group-data-[size=sm]/panel:min-h-8"><Text>Default panel</Text></View></Panel>
    </View>
  ), config.ui.theme);
}
