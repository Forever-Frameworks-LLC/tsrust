mod support;
#[test]
fn responsive_visibility_and_computed_have_current_diagnostics_and_hover(){
 let source="import {View,Text,signal,computed} from 'std:ui';function main(){const count=signal(1);const twice=computed<int>(()=>count.get()*2);const node=<View className=\"hidden lg:flex\"><Text>{twice.get()}</Text></View>;}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsrx",source);let diagnostics=session.wait_for_diagnostics("main.tsrx");assert!(diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");session.settle();
 for value in ["hidden","lg:flex","computed<int>"]{let hover=session.hover("main.tsrx",source,source.find(value).unwrap()+2).unwrap_or_default();assert!(!hover.is_empty(),"{value}");}
 let hover=session.hover("main.tsrx",source,source.find("hidden").unwrap()+2).unwrap();assert!(hover.contains("accessibility")&&hover.contains("ownership"),"{hover}");
}
