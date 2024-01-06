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
