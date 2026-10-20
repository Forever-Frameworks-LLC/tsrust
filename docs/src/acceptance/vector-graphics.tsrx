import { View, Text, Button, Svg, Path, document } from 'std:ui';
import { strictEqual, throws } from 'std:testing';

function main() {
  const html = document(<Svg aria-label="Saved" fill="none" stroke="currentColor" strokeWidth={2}><Path d="M2 12L10 20L22 4"/></Svg>, 'Vector');
  strictEqual(html.includes('role="img"'), true);
  strictEqual(html.includes('aria-label="Saved"'), true);
  strictEqual(html.includes('xmlns="http://www.w3.org/2000/svg"'), true);
  strictEqual(html.includes('d="M2 12L10 20L22 4"'), true);
  strictEqual(html.includes('stroke-width="2"'), true);
  const decorated = document(<Button><Svg className="size-4" fill="none" stroke="currentColor"><Path d="M4 12L9 17L20 6"/></Svg>Save</Button>, 'Button');
  strictEqual(decorated.includes('aria-hidden="true"'), true);
  const winding = document(<Svg fillRule="evenodd"><Path d="M0 0H24V24H0Z M6 6H18V18H6Z"/></Svg>, 'Hole');
  strictEqual(winding.includes('fill-rule="evenodd"'), true);
  throws(() => {
    const invalid = document(<Path d="M0 0L2 2"/>, 'Invalid');
  }, 'standalone Path must reject');
  throws(() => {
    const invalid = <Svg><Text>Wrong child</Text></Svg>;
  }, 'non-Path child must reject');
  throws(() => {
    const invalid = <View><Path d=""/></View>;
  }, 'Path under View must reject');
  throws(() => {
    const invalid = <Svg role="img" aria-hidden={false}/>;
  }, 'unlabelled image must reject');
  throws(() => {
    const invalid = <Svg viewBox="0 0 0 24"/>;
  }, 'empty viewBox must reject');
  throws(() => {
    const invalid = <Svg><Path d="M0"/></Svg>;
  }, 'incomplete path must reject');
  throws(() => {
    const invalid = <Svg width={Number.NaN}/>;
  }, 'NaN viewport must reject');
  throws(() => {
    const invalid = <Svg strokeWidth={Number.NaN}/>;
  }, 'NaN stroke must reject');
  throws(() => {
    const invalid = <Svg strokeMiterlimit={Number.POSITIVE_INFINITY}/>;
  }, 'infinite miter must reject');
  throws(() => {
    const invalid = <Svg><Path d="" strokeWidth={Number.NaN}/></Svg>;
  }, 'NaN path stroke must reject');
  throws(() => {
    const invalid = <Svg><Path d="" className="p-4"/></Svg>;
  }, 'Path layout class must reject');
  throws(() => {
    const invalid = <Svg className="w-auto"/>;
  }, 'intrinsic Svg sizing must reject');
  console.log('Vector graphics passed');
}
