mod support;
#[test]
fn empty_utilities_and_heading_theme_fields_have_current_documentation(){
 let source="import {View,Text,ThemeTypography} from 'std:ui';function main(){const typography:ThemeTypography={base:16,xl:24,headingFamily:'serif'};const node=<View className=\"border border-dashed max-w-sm mb-2 [&_svg]:size-4\"><Text className=\"text-sm/relaxed tracking-tight text-balance font-heading\">Create your first project</Text></View>;}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 for value in ["border-dashed","max-w-sm","mb-2","text-sm/relaxed","tracking-tight","text-balance","font-heading","headingFamily"]{let hover=session.hover("main.tsrx",source,source.find(value).unwrap()+2).unwrap_or_default();assert!(!hover.is_empty(),"{value}");}
 let counters=session.request("tsrust/counters",serde_json::Value::Null);for key in ["staleAnswers","emptyKnownReceiver","nonMemberAfterSeparator"]{assert_eq!(counters[key],0,"{counters}");}
}
