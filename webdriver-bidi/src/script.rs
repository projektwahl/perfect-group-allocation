use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::browsing_context::BrowsingContext;

/// <https://w3c.github.io/webdriver-bidi/#type-script-StackTrace>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StackTrace {
    call_frames: Vec<StackFrame>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-StackFrame>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    column_number: u64,
    function_name: String,
    line_number: u64,
    url: String,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-Source>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    realm: Realm,
    context: Option<BrowsingContext>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-Realm>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Realm(String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-Handle>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Handle(String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-InternalId>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InternalId(String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum RemoteValue {
    #[serde(rename = "undefined")]
    PrimitiveProtocolValueUndefined,
    #[serde(rename = "null")]
    PrimitiveProtocolValueNull,
    #[serde(rename = "string")]
    PrimitiveProtocolValueString {
        value: String,
    },
    #[serde(rename = "number")]
    PrimitiveProtocolValueNumber {
        value: f64, // TODO FIXME
    },
    #[serde(rename = "boolean")]
    PrimitiveProtocolValueBoolean {
        value: bool,
    },
    #[serde(rename = "bigint")]
    PrimitiveProtocolValueBigInt {
        value: String,
    },
    #[serde(rename = "symbol")]
    Symbol(SymbolRemoteValue),
    Array(ArrayRemoteValue),
    Object(ObjectRemoteValue),
    Function(FunctionRemoteValue),
    RegExp(RegExpRemoteValue),
    Date(DateRemoteValue),
    Map(MapRemoteValue),
    Set(SetRemoteValue),
    WeakMap(WeakMapRemoteValue),
    WeakSet(WeakSetRemoteValue),
    Iterator(IteratorRemoteValue),
    Generator(GeneratorRemoteValue),
    Error(ErrorRemoteValue),
    Proxy(ProxyRemoteValue),
    Promise(PromiseRemoteValue),
    TypedArray(TypedArrayRemoteValue),
    ArrayBuffer(ArrayBufferRemoteValue),
    NodeList(NodeListRemoteValue),
    HTMLCollection(HTMLCollectionRemoteValue),
    Node(NodeRemoteValue),
    WindowProxy(WindowProxyRemoteValue),
}

// TODO FIXME Option

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SymbolRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArrayRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: ListRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObjectRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: MappingRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FunctionRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegExpRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
} // .and script.RegExpLocalValue

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DateRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
} // .and script.DateLocalValue

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: MappingRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: ListRemoteValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WeakMapRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WeakSetRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IteratorRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProxyRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PromiseRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TypedArrayRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArrayBufferRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeListRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: Option<ListRemoteValue>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HTMLCollectionRemoteValue {
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
    value: Option<ListRemoteValue>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeRemoteValue {
    shared_id: SharedId,
    handle: Handle,
    internal_id: InternalId,
    value: NodeProperties,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
pub struct WindowProxyRemoteValue {
    value: WindowProxyProperties,
    handle: Option<Handle>,
    internal_id: Option<InternalId>,
}
