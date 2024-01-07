use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::browsing_context::BrowsingContext;

/// <https://w3c.github.io/webdriver-bidi/#type-script-StackTrace>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StackTrace {
    call_frames: Vec<StackFrame>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-StackFrame>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StackFrame {
    column_number: u64,
    function_name: String,
    line_number: u64,
    url: String,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-Source>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub realm: Realm,
    pub context: Option<BrowsingContext>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-Realm>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Realm(String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-Handle>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Handle(String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-InternalId>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct InternalId(String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-SharedId>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SharedId(String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum RemoteValue {
    #[serde(rename = "undefined")]
    PrimitiveProtocolValueUndefined,
    #[serde(rename = "null")]
    PrimitiveProtocolValueNull,
    #[serde(rename = "string")]
    PrimitiveProtocolValueString { value: String },
    #[serde(rename = "number")]
    PrimitiveProtocolValueNumber {
        value: f64, // TODO FIXME
    },
    #[serde(rename = "boolean")]
    PrimitiveProtocolValueBoolean { value: bool },
    #[serde(rename = "bigint")]
    PrimitiveProtocolValueBigInt { value: String },
    #[serde(rename = "symbol")]
    Symbol(SymbolRemoteValue),
    #[serde(rename = "array")]
    Array(ArrayRemoteValue),
    #[serde(rename = "object")]
    Object(ObjectRemoteValue),
    #[serde(rename = "function")]
    Function(FunctionRemoteValue),
    #[serde(rename = "regexp")]
    RegExp(RegExpRemoteValue),
    #[serde(rename = "date")]
    Date(DateRemoteValue),
    #[serde(rename = "map")]
    Map(MapRemoteValue),
    #[serde(rename = "set")]
    Set(SetRemoteValue),
    #[serde(rename = "weakmap")]
    WeakMap(WeakMapRemoteValue),
    #[serde(rename = "weakset")]
    WeakSet(WeakSetRemoteValue),
    #[serde(rename = "iterator")]
    Iterator(IteratorRemoteValue),
    #[serde(rename = "generator")]
    Generator(GeneratorRemoteValue),
    #[serde(rename = "error")]
    Error(ErrorRemoteValue),
    #[serde(rename = "proxy")]
    Proxy(ProxyRemoteValue),
    #[serde(rename = "promise")]
    Promise(PromiseRemoteValue),
    #[serde(rename = "typedarray")]
    TypedArray(TypedArrayRemoteValue),
    #[serde(rename = "arraybuffer")]
    ArrayBuffer(ArrayBufferRemoteValue),
    #[serde(rename = "nodelist")]
    NodeList(NodeListRemoteValue),
    #[serde(rename = "htmlcollection")]
    HTMLCollection(HTMLCollectionRemoteValue),
    #[serde(rename = "node")]
    Node(NodeRemoteValue),
    #[serde(rename = "window")]
    WindowProxy(WindowProxyRemoteValue),
}

// TODO FIXME Option

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ListRemoteValue(Vec<RemoteValue>);

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct MappingRemoteValue(Vec<Vec<(RemoteValue, RemoteValue)>>); // TODO FIXME

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SymbolRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArrayRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: ListRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ObjectRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: MappingRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct FunctionRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegExpRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: RegExpValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegExpValue {
    pattern: String,
    flags: Option<String>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DateRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: String,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct MapRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: MappingRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SetRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: ListRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WeakMapRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WeakSetRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IteratorRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct GeneratorRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ErrorRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ProxyRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PromiseRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct TypedArrayRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArrayBufferRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NodeListRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: Option<ListRemoteValue>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct HTMLCollectionRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: Option<ListRemoteValue>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NodeRemoteValue {
    shared_id: SharedId,
    handle: Handle,
    internal_id: InternalId,
    value: NodeProperties,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NodeProperties {
    node_type: u64,
    child_node_count: u64,
    attributes: Option<HashMap<String, String>>,
    children: Option<Vec<NodeRemoteValue>>,
    local_name: Option<String>,
    mode: Option<String>, // TODO open/closed
    #[serde(rename = "namespaceURI")]
    namespace_uri: Option<String>,
    node_value: Option<String>,
    shadow_root: Option<Box<NodeRemoteValue>>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WindowProxyRemoteValue {
    value: WindowProxyProperties,
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WindowProxyProperties {
    context: BrowsingContext,
}
