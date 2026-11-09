use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
const REFERENCES:&[(&str,&str,usize,&str,&str)]=&[
("HttpApp","get",0,"HTTP runtime > catalog-001",include_str!("../fixtures/http/runtime-cases/catalog-001.tsr")),
("HttpApp","post",0,"HTTP runtime > catalog-002",include_str!("../fixtures/http/runtime-cases/catalog-002.tsr")),
("HttpApp","put",0,"HTTP runtime > catalog-003",include_str!("../fixtures/http/runtime-cases/catalog-003.tsr")),
("HttpApp","patch",0,"HTTP runtime > catalog-004",include_str!("../fixtures/http/runtime-cases/catalog-004.tsr")),
("HttpApp","delete",0,"HTTP runtime > catalog-005",include_str!("../fixtures/http/runtime-cases/catalog-005.tsr")),
("HttpApp","head",0,"HTTP runtime > catalog-006",include_str!("../fixtures/http/runtime-cases/catalog-006.tsr")),
("HttpApp","options",0,"HTTP runtime > catalog-007",include_str!("../fixtures/http/runtime-cases/catalog-007.tsr")),
("HttpApp","route",0,"HTTP runtime > catalog-008",include_str!("../fixtures/http/runtime-cases/catalog-008.tsr")),
("HttpApp","use",0,"HTTP runtime > catalog-009",include_str!("../fixtures/http/runtime-cases/catalog-009.tsr")),
("HttpApp","fallback",0,"HTTP runtime > catalog-010",include_str!("../fixtures/http/runtime-cases/catalog-010.tsr")),
("HttpApp","listen",0,"HTTP runtime > catalog-011",include_str!("../fixtures/http/runtime-cases/catalog-011.tsr")),
("HttpApp","configure",0,"HTTP runtime > catalog-012",include_str!("../fixtures/http/runtime-cases/catalog-012.tsr")),
("HttpBody","next",0,"HTTP runtime > catalog-013",include_str!("../fixtures/http/runtime-cases/catalog-013.tsr")),
("HttpBody","text",0,"HTTP runtime > catalog-014",include_str!("../fixtures/http/runtime-cases/catalog-014.tsr")),
("HttpClient","send",0,"HTTP runtime > catalog-015",include_str!("../fixtures/http/runtime-cases/catalog-015.tsr")),
("HttpClientOptions","ca",0,"HTTP runtime > catalog-016",include_str!("../fixtures/http/runtime-cases/catalog-016.tsr")),
("HttpClientOptions","bodyLimit",0,"HTTP runtime > catalog-017",include_str!("../fixtures/http/runtime-cases/catalog-017.tsr")),
("HttpClientOptions","timeoutMs",0,"HTTP runtime > catalog-018",include_str!("../fixtures/http/runtime-cases/catalog-018.tsr")),
("HttpClientOptions","connectTimeoutMs",0,"HTTP runtime > catalog-019",include_str!("../fixtures/http/runtime-cases/catalog-019.tsr")),
("HttpClientOptions","followRedirects",0,"HTTP runtime > catalog-020",include_str!("../fixtures/http/runtime-cases/catalog-020.tsr")),
("HttpClientOptions","maxRedirects",0,"HTTP runtime > catalog-021",include_str!("../fixtures/http/runtime-cases/catalog-021.tsr")),
("HttpClientOptions","proxy",0,"HTTP runtime > catalog-022",include_str!("../fixtures/http/runtime-cases/catalog-022.tsr")),
("HttpClientOptions","noProxy",0,"HTTP runtime > catalog-023",include_str!("../fixtures/http/runtime-cases/catalog-023.tsr")),
("HttpContext","text",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpContext","html",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpContext","json",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpContext","header",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpContext","status",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpContext","set",0,"HTTP runtime > catalog-025",include_str!("../fixtures/http/runtime-cases/catalog-025.tsr")),
("HttpContext","get",0,"HTTP runtime > catalog-025",include_str!("../fixtures/http/runtime-cases/catalog-025.tsr")),
("HttpCorsOptions","origins",0,"HTTP runtime > catalog-026",include_str!("../fixtures/http/runtime-cases/catalog-026.tsr")),
("HttpCorsOptions","methods",0,"HTTP runtime > catalog-026",include_str!("../fixtures/http/runtime-cases/catalog-026.tsr")),
("HttpCorsOptions","allowedHeaders",0,"HTTP runtime > catalog-026",include_str!("../fixtures/http/runtime-cases/catalog-026.tsr")),
("HttpCorsOptions","exposeHeaders",0,"HTTP runtime > catalog-026",include_str!("../fixtures/http/runtime-cases/catalog-026.tsr")),
("HttpCorsOptions","credentials",0,"HTTP runtime > catalog-026",include_str!("../fixtures/http/runtime-cases/catalog-026.tsr")),
("HttpCorsOptions","maxAgeSeconds",0,"HTTP runtime > catalog-026",include_str!("../fixtures/http/runtime-cases/catalog-026.tsr")),
("HttpFormField","name",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpFormField","value",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpMultipartPart","name",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpMultipartPart","fileName",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpMultipartPart","contentType",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpMultipartPart","data",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpNext","run",0,"HTTP runtime > catalog-028",include_str!("../fixtures/http/runtime-cases/catalog-028.tsr")),
("HttpRequest","header",0,"HTTP runtime > catalog-029",include_str!("../fixtures/http/runtime-cases/catalog-029.tsr")),
("HttpRequest","setHeader",0,"HTTP runtime > catalog-030",include_str!("../fixtures/http/runtime-cases/catalog-030.tsr")),
("HttpRequest","body",0,"HTTP runtime > catalog-031",include_str!("../fixtures/http/runtime-cases/catalog-031.tsr")),
("HttpRequest","json",0,"HTTP runtime > catalog-032",include_str!("../fixtures/http/runtime-cases/catalog-032.tsr")),
("HttpRequest","text",0,"HTTP runtime > catalog-033",include_str!("../fixtures/http/runtime-cases/catalog-033.tsr")),
("HttpRequest","method",0,"HTTP runtime > catalog-034",include_str!("../fixtures/http/runtime-cases/catalog-034.tsr")),
("HttpRequest","url",0,"HTTP runtime > catalog-035",include_str!("../fixtures/http/runtime-cases/catalog-035.tsr")),
("HttpRequest","param",0,"HTTP runtime > catalog-036",include_str!("../fixtures/http/runtime-cases/catalog-036.tsr")),
("HttpRequest","form",0,"HTTP runtime > catalog-037",include_str!("../fixtures/http/runtime-cases/catalog-037.tsr")),
("HttpRequest","multipart",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpRequest","query",0,"HTTP runtime > catalog-038",include_str!("../fixtures/http/runtime-cases/catalog-038.tsr")),
("HttpRequest","queries",0,"HTTP runtime > catalog-039",include_str!("../fixtures/http/runtime-cases/catalog-039.tsr")),
("HttpResponse","header",0,"HTTP runtime > catalog-040",include_str!("../fixtures/http/runtime-cases/catalog-040.tsr")),
("HttpResponse","setHeader",0,"HTTP runtime > catalog-041",include_str!("../fixtures/http/runtime-cases/catalog-041.tsr")),
("HttpResponse","body",0,"HTTP runtime > catalog-042",include_str!("../fixtures/http/runtime-cases/catalog-042.tsr")),
("HttpResponse","text",0,"HTTP runtime > catalog-043",include_str!("../fixtures/http/runtime-cases/catalog-043.tsr")),
("HttpResponse","status",0,"HTTP runtime > catalog-044",include_str!("../fixtures/http/runtime-cases/catalog-044.tsr")),
("HttpRouter","get",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","post",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","put",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","patch",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","delete",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","head",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","options",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","route",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","use",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","listen",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","configure",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("HttpRouter","serveStatic",0,"HTTP runtime > catalog-045",include_str!("../fixtures/http/runtime-cases/catalog-045.tsr")),
("HttpRouter","webSocket",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpServer","address",0,"HTTP runtime > catalog-047",include_str!("../fixtures/http/runtime-cases/catalog-047.tsr")),
("HttpServer","shutdown",0,"HTTP runtime > catalog-011",include_str!("../fixtures/http/runtime-cases/catalog-011.tsr")),
("HttpServer","wait",0,"HTTP runtime > catalog-048",include_str!("../fixtures/http/runtime-cases/catalog-048.tsr")),
("HttpServerOptions","bodyLimit",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpServerOptions","maxConnections",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpServerOptions","maxRequests",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpServerOptions","requestTimeoutMs",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpServerOptions","shutdownTimeoutMs",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpServerOptions","handshakeTimeoutMs",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpServerOptions","requestsPerSecond",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("HttpSseEvent","data",0,"HTTP runtime > catalog-049",include_str!("../fixtures/http/runtime-cases/catalog-049.tsr")),
("HttpSseEvent","event",0,"HTTP runtime > catalog-050",include_str!("../fixtures/http/runtime-cases/catalog-050.tsr")),
("HttpSseEvent","id",0,"HTTP runtime > catalog-051",include_str!("../fixtures/http/runtime-cases/catalog-051.tsr")),
("HttpSseEvent","retryMs",0,"HTTP runtime > catalog-052",include_str!("../fixtures/http/runtime-cases/catalog-052.tsr")),
("HttpSseOptions","keepAliveMs",0,"HTTP runtime > catalog-053",include_str!("../fixtures/http/runtime-cases/catalog-053.tsr")),
("HttpSseOptions","keepAliveText",0,"HTTP runtime > catalog-054",include_str!("../fixtures/http/runtime-cases/catalog-054.tsr")),
("HttpWebSocket","receive",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocket","send",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocket","split",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocket","close",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocketBinary","kind",0,"HTTP runtime > catalog-055",include_str!("../fixtures/http/runtime-cases/catalog-055.tsr")),
("HttpWebSocketBinary","data",0,"HTTP runtime > catalog-056",include_str!("../fixtures/http/runtime-cases/catalog-056.tsr")),
("HttpWebSocketClientOptions","ca",0,"HTTP runtime > catalog-057",include_str!("../fixtures/http/runtime-cases/catalog-057.tsr")),
("HttpWebSocketClientOptions","headers",0,"HTTP runtime > catalog-058",include_str!("../fixtures/http/runtime-cases/catalog-058.tsr")),
("HttpWebSocketClientOptions","socket",0,"HTTP runtime > catalog-059",include_str!("../fixtures/http/runtime-cases/catalog-059.tsr")),
("HttpWebSocketClose","kind",0,"HTTP runtime > catalog-060",include_str!("../fixtures/http/runtime-cases/catalog-060.tsr")),
("HttpWebSocketClose","code",0,"HTTP runtime > catalog-061",include_str!("../fixtures/http/runtime-cases/catalog-061.tsr")),
("HttpWebSocketClose","reason",0,"HTTP runtime > catalog-062",include_str!("../fixtures/http/runtime-cases/catalog-062.tsr")),
("HttpWebSocketHalves","read",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocketHalves","write",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocketHeader","name",0,"HTTP runtime > catalog-063",include_str!("../fixtures/http/runtime-cases/catalog-063.tsr")),
("HttpWebSocketHeader","value",0,"HTTP runtime > catalog-064",include_str!("../fixtures/http/runtime-cases/catalog-064.tsr")),
("HttpWebSocketOptions","readBufferSize",0,"HTTP runtime > catalog-065",include_str!("../fixtures/http/runtime-cases/catalog-065.tsr")),
("HttpWebSocketOptions","writeBufferSize",0,"HTTP runtime > catalog-066",include_str!("../fixtures/http/runtime-cases/catalog-066.tsr")),
("HttpWebSocketOptions","maxWriteBufferSize",0,"HTTP runtime > catalog-067",include_str!("../fixtures/http/runtime-cases/catalog-067.tsr")),
("HttpWebSocketOptions","maxMessageSize",0,"HTTP runtime > catalog-068",include_str!("../fixtures/http/runtime-cases/catalog-068.tsr")),
("HttpWebSocketOptions","maxFrameSize",0,"HTTP runtime > catalog-069",include_str!("../fixtures/http/runtime-cases/catalog-069.tsr")),
("HttpWebSocketOptions","acceptUnmaskedFrames",0,"HTTP runtime > catalog-070",include_str!("../fixtures/http/runtime-cases/catalog-070.tsr")),
("HttpWebSocketRead","receive",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocketText","kind",0,"HTTP runtime > catalog-071",include_str!("../fixtures/http/runtime-cases/catalog-071.tsr")),
("HttpWebSocketText","data",0,"HTTP runtime > catalog-072",include_str!("../fixtures/http/runtime-cases/catalog-072.tsr")),
("HttpWebSocketWrite","send",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("HttpWebSocketWrite","close",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("std:http","Router",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("std:http","Context",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("std:http","ContextNext",0,"HTTP runtime > catalog-024",include_str!("../fixtures/http/runtime-cases/catalog-024.tsr")),
("std:http","App",0,"HTTP runtime > catalog-073",include_str!("../fixtures/http/runtime-cases/catalog-073.tsr")),
("std:http","Client",0,"HTTP runtime > catalog-074",include_str!("../fixtures/http/runtime-cases/catalog-074.tsr")),
("std:http","text",0,"HTTP runtime > catalog-075",include_str!("../fixtures/http/runtime-cases/catalog-075.tsr")),
("std:http","html",0,"HTTP runtime > catalog-076",include_str!("../fixtures/http/runtime-cases/catalog-076.tsr")),
("std:http","json",0,"HTTP runtime > catalog-077",include_str!("../fixtures/http/runtime-cases/catalog-077.tsr")),
("std:http","request",0,"HTTP runtime > catalog-078",include_str!("../fixtures/http/runtime-cases/catalog-078.tsr")),
("std:http","stream",0,"HTTP runtime > catalog-079",include_str!("../fixtures/http/runtime-cases/catalog-079.tsr")),
("std:http","Request",0,"HTTP runtime > catalog-080",include_str!("../fixtures/http/runtime-cases/catalog-080.tsr")),
("std:http","Response",0,"HTTP runtime > catalog-081",include_str!("../fixtures/http/runtime-cases/catalog-081.tsr")),
("std:http","Body",0,"HTTP runtime > catalog-082",include_str!("../fixtures/http/runtime-cases/catalog-082.tsr")),
("std:http","Server",0,"HTTP runtime > catalog-083",include_str!("../fixtures/http/runtime-cases/catalog-083.tsr")),
("std:http","Next",0,"HTTP runtime > catalog-084",include_str!("../fixtures/http/runtime-cases/catalog-084.tsr")),
("std:http","HttpError",0,"HTTP runtime > catalog-085",include_str!("../fixtures/http/runtime-cases/catalog-085.tsr")),
("std:http","TlsOptions",0,"HTTP runtime > catalog-086",include_str!("../fixtures/http/runtime-cases/catalog-086.tsr")),
("std:http","ClientOptions",0,"HTTP runtime > catalog-087",include_str!("../fixtures/http/runtime-cases/catalog-087.tsr")),
("std:http","ServerOptions",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("std:http","ServerOptions",1,"HTTP runtime > catalog-088",include_str!("../fixtures/http/runtime-cases/catalog-088.tsr")),
("std:http","FormField",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("std:http","FormField",1,"HTTP runtime > catalog-089",include_str!("../fixtures/http/runtime-cases/catalog-089.tsr")),
("std:http","MultipartPart",0,"HTTP runtime > catalog-027",include_str!("../fixtures/http/runtime-cases/catalog-027.tsr")),
("std:http","MultipartPart",1,"HTTP runtime > catalog-090",include_str!("../fixtures/http/runtime-cases/catalog-090.tsr")),
("std:http","getCookie",0,"HTTP runtime > catalog-025",include_str!("../fixtures/http/runtime-cases/catalog-025.tsr")),
("std:http","setCookie",0,"HTTP runtime > catalog-025",include_str!("../fixtures/http/runtime-cases/catalog-025.tsr")),
("std:http","deleteCookie",0,"HTTP runtime > catalog-025",include_str!("../fixtures/http/runtime-cases/catalog-025.tsr")),
("std:http","CookieOptions",0,"HTTP runtime > catalog-025",include_str!("../fixtures/http/runtime-cases/catalog-025.tsr")),
("std:http","streamChannel",0,"HTTP runtime > catalog-091",include_str!("../fixtures/http/runtime-cases/catalog-091.tsr")),
("std:http","requestBytes",0,"HTTP runtime > catalog-092",include_str!("../fixtures/http/runtime-cases/catalog-092.tsr")),
("std:http","requestStream",0,"HTTP runtime > catalog-093",include_str!("../fixtures/http/runtime-cases/catalog-093.tsr")),
("std:http","requestChannel",0,"HTTP runtime > catalog-094",include_str!("../fixtures/http/runtime-cases/catalog-094.tsr")),
("std:http","requestForm",0,"HTTP runtime > catalog-095",include_str!("../fixtures/http/runtime-cases/catalog-095.tsr")),
("std:http","requestMultipart",0,"HTTP runtime > catalog-096",include_str!("../fixtures/http/runtime-cases/catalog-096.tsr")),
("std:http","Middleware",0,"HTTP runtime > catalog-097",include_str!("../fixtures/http/runtime-cases/catalog-097.tsr")),
("std:http","CorsOptions",0,"HTTP runtime > catalog-098",include_str!("../fixtures/http/runtime-cases/catalog-098.tsr")),
("std:http","cors",0,"HTTP runtime > catalog-099",include_str!("../fixtures/http/runtime-cases/catalog-099.tsr")),
("std:http","secureHeaders",0,"HTTP runtime > catalog-100",include_str!("../fixtures/http/runtime-cases/catalog-100.tsr")),
("std:http","compress",0,"HTTP runtime > catalog-101",include_str!("../fixtures/http/runtime-cases/catalog-101.tsr")),
("std:http","accessLog",0,"HTTP runtime > catalog-102",include_str!("../fixtures/http/runtime-cases/catalog-102.tsr")),
("std:http","SseEvent",0,"HTTP runtime > catalog-049",include_str!("../fixtures/http/runtime-cases/catalog-049.tsr")),
("std:http","SseOptions",0,"HTTP runtime > catalog-053",include_str!("../fixtures/http/runtime-cases/catalog-053.tsr")),
("std:http","WebSocketOptions",0,"HTTP runtime > catalog-065",include_str!("../fixtures/http/runtime-cases/catalog-065.tsr")),
("std:http","WebSocketClientOptions",0,"HTTP runtime > catalog-057",include_str!("../fixtures/http/runtime-cases/catalog-057.tsr")),
("std:http","WebSocketHeader",0,"HTTP runtime > catalog-063",include_str!("../fixtures/http/runtime-cases/catalog-063.tsr")),
("std:http","WebSocketText",0,"HTTP runtime > catalog-071",include_str!("../fixtures/http/runtime-cases/catalog-071.tsr")),
("std:http","WebSocketBinary",0,"HTTP runtime > catalog-055",include_str!("../fixtures/http/runtime-cases/catalog-055.tsr")),
("std:http","WebSocketClose",0,"HTTP runtime > catalog-060",include_str!("../fixtures/http/runtime-cases/catalog-060.tsr")),
("std:http","WebSocketHalves",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("std:http","WebSocket",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("std:http","WebSocketRead",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("std:http","WebSocketWrite",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("std:http","WebSocketMessage",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
("std:http","sse",0,"HTTP runtime > catalog-103",include_str!("../fixtures/http/runtime-cases/catalog-103.tsr")),
("std:http","sseWithProducer",0,"HTTP runtime > catalog-103",include_str!("../fixtures/http/runtime-cases/catalog-103.tsr")),
("std:http","connectWebSocket",0,"HTTP runtime > catalog-046",include_str!("../fixtures/http/runtime-cases/catalog-046.tsr")),
];
const REGRESSIONS:&[(&str,&str,&str,&str)]=&[
("loopback_routes_middleware_streaming_and_cleanup_execute","HTTP runtime > loopback","200\n{\"id\":\"42\",\"message\":\"Hello\"}\nhello\n",include_str!("../fixtures/http/runtime-cases/loopback.tsr")),
("validated_json_body_executes","HTTP runtime > validated-body","201\n{\"count\":2,\"name\":\"Pencil\"}\n400\n",include_str!("../fixtures/http/runtime-cases/validated-body.tsr")),
("consecutive_chunk_reads_release_the_receiver_borrow","HTTP runtime > consecutive-chunks","3\ntrue\ndone\n404\n",include_str!("../fixtures/http/runtime-cases/consecutive-chunks.tsr")),
("tls_configuration_errors_are_typed","HTTP runtime > tls-errors","",include_str!("../fixtures/http/runtime-cases/tls-errors.tsr")),
("producer_stream_executes_and_rejects_wrong_payload_or_reuse","HTTP runtime > producer-stream","",include_str!("../fixtures/http/runtime-cases/producer-stream.tsr")),
("query_helpers_preserve_values_and_body_ownership","HTTP runtime > query-values","",include_str!("../fixtures/http/runtime-cases/query-values.tsr")),
("query_router_example","HTTP runtime > query-router","one two, three\n",include_str!("../fixtures/http/runtime-cases/query-router.tsr")),
];
fn exported_main(source:&str)->String{
 let tokens=tsrust_compiler::documentation::highlight(source,"tsrust").expect("valid authored HTTP example").into_iter().filter(|token|!matches!(token.class.as_str(),"plain"|"comment")).collect::<Vec<_>>();
 let declarations=tokens.windows(3).enumerate().filter(|(_,tokens)|tokens[0].class=="keyword"&&tokens[0].text=="function"&&tokens[1].class=="name"&&tokens[1].text=="main"&&tokens[2].text=="(").map(|(index,_)|index).collect::<Vec<_>>();
 assert_eq!(declarations.len(),1,"one authored main declaration");let mut index=declarations[0];
 if index>0&&tokens[index-1].class=="keyword"&&tokens[index-1].text=="async"{index-=1;}
 assert!(index==0||tokens[index-1].text!="export","authored example is not already exported");
 let mut module=source.to_owned();module.insert_str(tokens[index].source.start,"export ");module
}
fn assert_exported_source(authored:&str,fixture:&str,case:&str){
 assert_eq!(super::support::format_round_trip(&exported_main(authored)),super::support::format_round_trip(fixture),"source differs beyond added export/formatting: {case}");
}
fn catalog()->Value{serde_json::from_str(include_str!("../../../tsrust-lsp/src/builtins.json")).unwrap()}
fn example<'a>(catalog:&'a Value,family:&str,name:&str,index:usize)->&'a Value{&catalog[family].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap()["examples"][index]}
pub fn assert_regression_source(owner:&str,source:&str){let rows=REGRESSIONS.iter().filter(|row|row.0==owner).collect::<Vec<_>>();assert_eq!(rows.len(),1,"missing or duplicate original HTTP case: {owner}");assert_exported_source(source,rows[0].3,rows[0].1);}
pub fn assert_catalog_sources(){
 let catalog=catalog();let mut actual=BTreeSet::new();let mut unique=BTreeSet::new();let mut mapped_cases=BTreeMap::new();
 for(family,entries)in catalog.as_object().unwrap(){if family!="std:http"&&!family.starts_with("Http"){continue;}for entry in entries.as_array().unwrap(){for(index,_)in entry["examples"].as_array().into_iter().flatten().enumerate(){actual.insert((family.as_str(),entry["name"].as_str().unwrap(),index));}}}
 let mapped=REFERENCES.iter().map(|&(family,name,index,_,_)|(family,name,index)).collect::<BTreeSet<_>>();assert_eq!(actual,mapped);
 for &(family,name,index,case,source)in REFERENCES{let example=example(&catalog,family,name,index);let raw=example["source"].as_str().unwrap();assert_exported_source(raw,source,case);let key=(super::support::format_round_trip(raw),example["output"].as_str().unwrap());unique.insert(key.clone());if let Some(previous)=mapped_cases.insert(case,key){assert_eq!(previous,(super::support::format_round_trip(raw),example["output"].as_str().unwrap()));}}
 assert_eq!(actual.len(),172);assert_eq!(unique.len(),103);assert_eq!(mapped_cases.len(),unique.len());
}
pub fn assert_catalog_case(family:&str,name:&str,index:usize){let catalog=catalog();let rows=REFERENCES.iter().filter(|row|row.0==family&&row.1==name&&row.2==index).collect::<Vec<_>>();assert_eq!(rows.len(),1);assert_exported_source(example(&catalog,family,name,index)["source"].as_str().unwrap(),rows[0].4,rows[0].3);}
pub fn assert_sources(){
 assert_catalog_sources();
 let mut catalog_modules=BTreeMap::new();for &(_,_,_,case,source)in REFERENCES{if let Some(previous)=catalog_modules.insert(case,source){assert_eq!(previous,source);}}
 let mut modules=catalog_modules.into_iter().collect::<Vec<_>>();for &(_,case,_,source)in REGRESSIONS{super::support::format_round_trip(source);modules.push((case,source));}
 let mut suite="import {describe,test} from 'std:testing';\n".to_owned();
 for(index,(case,_))in modules.iter().enumerate(){let name=case.strip_prefix("HTTP runtime > ").unwrap();suite.push_str(&format!("import {{main as case{index}}} from './runtime-cases/{name}.tsr';\n"));}
 suite.push_str("describe('HTTP runtime',()=>{\n");
 for(index,(case,source))in modules.iter().enumerate(){let name=case.strip_prefix("HTTP runtime > ").unwrap();let asynchronous=source.contains("export async function main(");suite.push_str(&format!("test('{name}',{}()=>{{{}case{index}();}});\n",if asynchronous{"async "}else{""},if asynchronous{"await "}else{""}));}
 suite.push_str("});\n");assert_eq!(super::support::format_round_trip(&suite),super::support::format_round_trip(include_str!("../fixtures/http/runtime.test.tsr")),"registered HTTP source/call correspondence changed");
}
pub fn assert_report(output:std::process::Output){
 super::support::success(&output);let stderr=String::from_utf8(output.stderr).unwrap();
 if !stderr.is_empty(){
  eprint!("{stderr}");assert_eq!(stderr.matches("warning[").count(),4,"{stderr}");let warnings=stderr.split("warning[unused-task]:").skip(1).collect::<Vec<_>>();assert_eq!(warnings.len(),4,"{stderr}");
  assert_eq!(warnings.iter().filter(|warning|warning.contains("catalog-046.tsr")).count(),1,"{stderr}");assert_eq!(warnings.iter().filter(|warning|warning.contains("consecutive-chunks.tsr")).count(),3,"{stderr}");assert!(stderr.starts_with("warning[unused-task]:")&&!stderr.contains("error["),"{stderr}");
 }
 let text=String::from_utf8(output.stdout).unwrap();print!("{text}");let rows=text.lines().map(|line|serde_json::from_str::<Value>(line).unwrap_or_else(|error|panic!("invalid JSON {line}: {error}"))).collect::<Vec<_>>();
 let catalog=catalog();let mut expected=BTreeMap::new();for &(family,name,index,case,_)in REFERENCES{let output=example(&catalog,family,name,index)["output"].as_str().unwrap();if let Some(previous)=expected.insert(case,output){assert_eq!(previous,output);}}
 for &(_,case,output,_)in REGRESSIONS{assert!(expected.insert(case,output).is_none());}
 assert_eq!(expected.len(),110);

 let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),expected.len());assert_eq!(tests.iter().map(|row|row["name"].as_str().unwrap()).collect::<BTreeSet<_>>(),expected.keys().copied().collect());
 for row in &rows {match row["event"].as_str().expect("event name"){
 "test"=>{assert_eq!(row["status"],"passed","{row}");let ms=row["durationMs"].as_f64().unwrap();assert!(ms.is_finite()&&ms>=0.0);},
 "output"=>{assert_eq!(row["stream"],"stdout","{row}");assert!(expected.contains_key(row["name"].as_str().unwrap()),"unknown/unattributed output: {row}");},
 "build"|"file"|"summary"=>{},other=>panic!("unexpected reporter event: {other}: {row}"),}}
 for(case,expected)in &expected{let actual=rows.iter().filter(|row|row["event"]=="output"&&row["name"]==*case).map(|row|row["text"].as_str().unwrap()).collect::<String>();assert_eq!(actual,*expected,"{case}");}
 let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);for field in ["durationMs","checkMs","emitMs","backendMs","cacheMs","nativeBuildMs"]{let ms=builds[0][field].as_f64().unwrap();assert!(ms.is_finite()&&ms>=0.0);}
 let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);assert_eq!(rows.last().unwrap()["event"],"summary");for(field,value)in[("passed",expected.len()as u64),("failed",0),("hookFailures",0),("skipped",0),("filesPassed",1),("filesFailed",0)]{assert_eq!(summaries[0][field],value,"{field}");}
}
