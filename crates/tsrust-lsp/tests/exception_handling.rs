use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;

#[test]
fn exception_source_formats_and_keeps_editor_symbols() {
    let source = "function value():int{try{try{return 4;}finally{console.log(5);}}finally{console.log(6);}}function main(){try{console.log(value());}catch(error){console.log(error);}}";
    tsrust_compiler::check(source).unwrap();
    let formatted = tsrust_compiler::format(source).unwrap();
    assert!(formatted.contains("} finally {"), "{formatted}");
    assert!(formatted.contains("} catch (error) {"), "{formatted}");
    tsrust_compiler::check(&formatted).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    let symbols = SourceIndex::new(&formatted)
        .request("textDocument/documentSymbol", &json!({}), "file:///exception.tsr")
        .unwrap();
    assert_eq!(symbols.as_array().unwrap().len(), 2);
}

#[test]
fn catch_binding_has_hover_definition_and_scope() {
    let source = "function risky(flag:boolean):int{if(flag){throw \"bad\";}return 1;}function main(){try{console.log(risky(true));}catch(error){if(typeof error===\"string\"){console.log(error.length);}}finally{console.log(0);}}";
    tsrust_compiler::check(source).unwrap();
    let position = |needle: &str| {
        let offset = source.find(needle).unwrap();
        json!({"line":0,"character":source[..offset].encode_utf16().count()})
    };
    let ask = |method: &str, needle: &str| {
        SourceIndex::new(source)
            .request(method, &json!({"position":position(needle)}), "file:///exception.tsr")
            .unwrap()
    };
    let declared = ask("textDocument/hover", "error){");
    assert!(declared["contents"]["value"].as_str().unwrap_or("").contains("(catch) error"), "{declared}");
    let used = ask("textDocument/hover", "error.length");
    assert!(used["contents"]["value"].as_str().unwrap_or("").contains("error"), "{used}");
    let definition = ask("textDocument/definition", "error.length");
    assert_eq!(definition["range"]["start"]["character"], position("error){")["character"], "{definition}");
    let members = ask("textDocument/hover", "length);}}");
    assert!(!members.is_null(), "narrowed string member has documentation");
    // The name is not visible after the handler.
    let outside = "function main(){try{throw 1;}catch(error){console.log(0);}console.log(error);}";
    let offset = outside.rfind("error").unwrap();
    let after = SourceIndex::new(outside)
        .request("textDocument/definition", &json!({"position":{"line":0,"character":offset}}), "file:///exception.tsr")
        .unwrap();
    assert!(after.is_null(), "{after}");
}

#[test]
fn checked_array_callback_formats_and_preserves_catch_hover(){
 let source="function main(){const values=[1,2];try{values.map((value):int=>{throw \"bad\";});}catch(error){if(typeof error===\"string\"){console.log(error.length);}}finally{console.log(0);}}";
 tsrust_compiler::check(source).unwrap();
 let formatted=tsrust_compiler::format(source).unwrap();
 tsrust_compiler::check(&formatted).unwrap();
 assert!(formatted.contains("} catch (error) {"));assert!(formatted.contains("} finally {"));
 let offset=source.find("error.length").unwrap();
 let hover=SourceIndex::new(source).request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),"file:///callback.tsr").unwrap();
 assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("error"),"{hover}");
}
