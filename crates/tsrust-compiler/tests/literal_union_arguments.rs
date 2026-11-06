mod support;

fn native(source:&str){let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path),180);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());}

#[test]
fn finite_string_arguments_keep_required_optional_and_stored_context(){native(r#"
import {strictEqual} from 'std:testing';
type Phase='ready'|'done';
function inspect(value:ReadOnly<Phase>):int{return value.length;}
function optional(value?:ReadOnly<Phase>):int{if(value===undefined){return 0;}return value==='ready'?5:4;}
type Actions={required:Function<int,[value:ReadOnly<Phase>]>;optional:Function<int,[value?:ReadOnly<Phase>]>;};
function main(){
 const alias=inspect;const optionalAlias=optional;const actions:Actions={required:inspect,optional};
 strictEqual(inspect('ready'),5);strictEqual(optional('done'),4);strictEqual(optional(),0);strictEqual(optional(undefined),0);
 strictEqual(alias('done'),4);strictEqual(optionalAlias('ready'),5);strictEqual(actions.required('ready'),5);strictEqual(actions.optional('done'),4);strictEqual(actions.optional(),0);
 const atom:'ready'='ready';const phase:Phase='done';let i=0;while(i<3){strictEqual(inspect(atom),5);strictEqual(optional(atom),5);strictEqual(actions.required(phase),4);strictEqual(actions.optional(phase),4);i++;}
 strictEqual(atom==='ready',true);strictEqual(phase.length,4);
}
"#);}

#[test]
fn finite_string_arguments_still_reject_unknown_literals_and_widened_strings(){
 let declarations="type Phase='ready'|'done';function inspect(value:ReadOnly<Phase>):int{return value.length;}function optional(value?:ReadOnly<Phase>):int{if(value===undefined){return 0;}return value==='ready'?5:4;}type Actions={required:Function<int,[value:ReadOnly<Phase>]>;optional:Function<int,[value?:ReadOnly<Phase>]>;};";
 for body in["inspect('missing');","optional('missing');","const alias=inspect;alias('missing');","const actions:Actions={required:inspect,optional};actions.required('missing');","const actions:Actions={required:inspect,optional};actions.optional('missing');","const text:string='ready';inspect(text);","const text:string='done';optional(text);"]{let source=format!("{declarations}function main(){{{body}}}");assert!(tsrust_compiler::check(&support::format_round_trip(&source)).is_err(),"accepted finite-union widening: {source}");}
}

#[test]
fn concrete_int_decimal_members_remain_borrowed_through_required_unions(){native(r#"
import {parseLocale,createNumberFormat,NumberFormat} from 'std:locale';import {Decimal,parseDecimal,format as formatDecimal} from 'std:decimal';import {strictEqual} from 'std:testing';
function width(formatter:ReadOnly<NumberFormat>,value:ReadOnly<int|Decimal>):int{return formatter.format(value).length;}
function main(){const formatter=unwrap(createNumberFormat(unwrap(parseLocale('en'))));const decimal=unwrap(parseDecimal('2.345'));const tagged:int|Decimal=unwrap(parseDecimal('1.25'));const alias=width;let i=0;while(i<3){strictEqual(width(formatter,decimal),5);strictEqual(width(formatter,7),1);strictEqual(alias(formatter,decimal),5);strictEqual(width(formatter,tagged),4);i++;}strictEqual(formatDecimal(decimal),'2.345');}
"#);}

#[test]
fn required_record_literal_and_factory_union_context_is_preserved(){native(r#"
import {strictEqual} from 'std:testing';type Left={kind:'left';count:int;};type Right={kind:'right';label:string;};
function inspect(value:ReadOnly<Left|Right>):int{if(value.kind==='left'){return value.count;}return value.label.length;}
function make<T>(left:boolean):T{if(left){return {kind:'left',count:7};}return {kind:'right',label:'done'};}
function main(){strictEqual(inspect({kind:'left',count:7}),7);strictEqual(inspect({kind:'right',label:'done'}),4);strictEqual(inspect(make(true)),7);}
"#);}


#[test]
fn guarded_optional_finite_strings_share_observations_without_moving_owners(){native(r#"
import {strictEqual} from 'std:testing';
import {SeekOrigin as Origin} from 'std:fs';
type Phase='request-start'|'task-end';type Alias=Phase;
function width(value?:ReadOnly<Alias>):int{
 if(value===undefined){return 0;}
 strictEqual(value.includes('-'),true);
 strictEqual(value[0],value.startsWith('request')?'r':'t');
 strictEqual(value.toUpperCase().length,value.length);
 return value.length;
}
function forward(value?:ReadOnly<Alias>):int{return width(value);}
function textOnly(value?:ReadOnly<Alias|int>):int{
 if(value===undefined){return 0;}if(typeof value==='number'){return -1;}return value.length;
}
function originWidth(value?:ReadOnly<Origin>):int{if(value===undefined){return 0;}return value.length;}
function guardedLoop(value?:ReadOnly<Alias>):int{
 let total=0;let n=0;while(n<3){if(value===undefined){return 0;}total+=value.length;n++;}return total;
}
function main(){
 strictEqual(width(),0);strictEqual(width(undefined),0);strictEqual(width('request-start'),13);strictEqual(width('task-end'),8);
 const inspect=width;const phase:Alias='task-end';let n=0;
 while(n<3){strictEqual(inspect(phase),8);strictEqual(forward(phase),8);strictEqual(textOnly(phase),8);n++;}
 strictEqual(textOnly(7),-1);strictEqual(textOnly(),0);strictEqual(phase.length,8);
 strictEqual(originWidth(),0);const origin:Origin='current';strictEqual(originWidth(origin),7);strictEqual(originWidth(origin),7);strictEqual(origin.length,7);
 strictEqual(guardedLoop(phase),24);strictEqual(guardedLoop(),0);
}
"#);}

#[test]
fn optional_finite_string_observations_keep_absence_mixed_and_loop_guards(){
 let prefix="type Phase='request-start'|'task-end';";
 for(body,code)in[
  ("function inspect(value?:ReadOnly<Phase>):int{return value.length;}function main(){inspect();}","type"),
  ("function inspect(value?:ReadOnly<Phase|int>):int{if(value===undefined){return 0;}return value.length;}function main(){inspect(7);}","type"),
  ("function inspect(initial?:Phase):int{let value:Phase|undefined=initial;if(value===undefined){return 0;}value=undefined;return value.length;}function main(){inspect('task-end');}","type"),
  ("function inspect(initial?:Phase):int{let value:Phase|undefined=initial;if(value===undefined){return 0;}let n=0;while(n<2){console.log(value.length);value=undefined;n++;}return 0;}function main(){inspect('task-end');}","loop-narrowing"),
 ]{let source=support::format_round_trip(&format!("{prefix}{body}"));let error=tsrust_compiler::check(&source).err().expect("invalid observation must fail");assert_eq!(error.code,code,"{source}: {error:?}");}
 let source=support::format_round_trip(&format!("{prefix}function inspect(value?:ReadOnly<Phase>):void{{if(value===undefined){{return;}}value.length=1;}}function main(){{inspect('task-end');}}"));
 assert!(tsrust_compiler::check(&source).is_err(),"readonly length became writable");
}
