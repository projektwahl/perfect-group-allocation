use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::browsing_context::BrowsingContext;

/// <https://w3c.github.io/webdriver-bidi/#type-script-StackTrace>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StackTrace {
    pub call_frames: Vec<StackFrame>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-StackFrame>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StackFrame {
    pub column_number: u64,
    pub function_name: String,
    pub line_number: u64,
    pub url: String,
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
pub struct Realm(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-Handle>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Handle(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-InternalId>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct InternalId(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-SharedId>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SharedId(pub String);

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
pub struct ListRemoteValue(pub Vec<RemoteValue>);

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct MappingRemoteValue(pub Vec<Vec<(RemoteValue, RemoteValue)>>); // TODO FIXME

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SymbolRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArrayRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: ListRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ObjectRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: MappingRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct FunctionRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegExpRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: RegExpValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegExpValue {
    pub pattern: String,
    pub flags: Option<String>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DateRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: String,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct MapRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: MappingRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SetRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: ListRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WeakMapRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WeakSetRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct IteratorRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct GeneratorRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ErrorRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ProxyRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PromiseRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct TypedArrayRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArrayBufferRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NodeListRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: Option<ListRemoteValue>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct HTMLCollectionRemoteValue {
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
    pub value: Option<ListRemoteValue>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NodeRemoteValue {
    pub shared_id: SharedId,
    pub handle: Handle,
    pub internal_id: InternalId,
    pub value: NodeProperties,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NodeProperties {
    pub node_type: u64,
    pub child_node_count: u64,
    pub attributes: Option<HashMap<String, String>>,
    pub children: Option<Vec<NodeRemoteValue>>,
    pub local_name: Option<String>,
    pub mode: Option<String>, // TODO open/closed
    #[serde(rename = "namespaceURI")]
    pub namespace_uri: Option<String>,
    pub node_value: Option<String>,
    pub shadow_root: Option<Box<NodeRemoteValue>>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WindowProxyRemoteValue {
    pub value: WindowProxyProperties,
    pub handle: Option<Handle>,
    pub internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WindowProxyProperties {
    pub context: BrowsingContext,
}
