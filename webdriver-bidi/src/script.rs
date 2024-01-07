//! <https://w3c.github.io/webdriver-bidi/#module-script>

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::browsing_context::BrowsingContext;
use crate::protocol::Extensible;

/// The script.Channel type represents the id of a specific channel used to send custom messages from the remote end to the local end.
///
/// <https://w3c.github.io/webdriver-bidi/#type-script-Channel>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Channel(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-ChannelValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "channel")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ChannelValue {
    pub value: ChannelProperties,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-ChannelValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "channel")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ChannelProperties {
    pub channel: Channel,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub serialization_options: Option<SerializationOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ownership: Option<ResultOwnership>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-EvaluateResult>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum EvaluateResult {
    Success(EvaluateResultSuccess),
    Exception(EvaluateResultException),
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-EvaluateResult>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "success")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EvaluateResultSuccess {
    result: RemoteValue,
    realm: Realm,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-EvaluateResult>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "exception")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EvaluateResultException {
    exception_details: ExceptionDetails,
    realm: Realm,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-ExceptionDetails>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ExceptionDetails {
    pub column_number: u64,
    pub exception: RemoteValue,
    pub line_number: u64,
    pub stack_trace: StackTrace,
    pub text: String,
}

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

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum LocalValue {
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
    #[serde(rename = "channel")]
    Channel(ChannelValue),
    #[serde(rename = "array")]
    Array(ArrayLocalValue),
    #[serde(rename = "date")]
    Date(DateLocalValue),
    #[serde(rename = "map")]
    Map(MapLocalValue),
    #[serde(rename = "object")]
    Object(ObjectLocalValue),
    #[serde(rename = "regexp")]
    RegExp(RegExpLocalValue),
    #[serde(rename = "set")]
    Set(SetLocalValue),
    #[serde(untagged)]
    RemoteReference(RemoteReference),
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ListLocalValue(Vec<LocalValue>);

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ArrayLocalValue {
    value: ListLocalValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DateLocalValue {
    value: String,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
// TODO FIXME first value may be a string
pub struct MappingLocalValue(Vec<(LocalValue, LocalValue)>);

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct MapLocalValue {
    value: MappingLocalValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ObjectLocalValue {
    value: MappingLocalValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegExpValue {
    pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    flags: Option<String>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegExpLocalValue {
    value: RegExpValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-LocalValue>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SetLocalValue {
    value: ListLocalValue,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-PreloadScript>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PreloadScript(pub String);

/// <https://w3c.github.io/webdriver-bidi/#type-script-Realm>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Realm(pub String);

// TODO FIXME https://w3c.github.io/webdriver-bidi/#type-script-PrimitiveProtocolValue?

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RealmInfo {
    realm: Realm,
    origin: String,
    #[serde(flatten)]
    inner: RealmInfoInner,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum RealmInfoInner {
    #[serde(rename = "window")]
    Window(WindowRealmInfo),
    #[serde(rename = "dedicated-worker")]
    DedicatedWorker(DedicatedWorkerRealmInfo),
    #[serde(rename = "shared-worker")]
    SharedWorker(SharedWorkerRealmInfo),
    #[serde(rename = "service-worker")]
    ServiceWorker(ServiceWorkerRealmInfo),
    #[serde(rename = "worker")]
    Worker(WorkerRealmInfo),
    #[serde(rename = "paint-worklet")]
    PaintWorklet(PaintWorkletRealmInfo),
    #[serde(rename = "audio-worklet")]
    AudioWorklet(AudioWorkletRealmInfo),
    #[serde(rename = "worklet")]
    Worklet(WorkletRealmInfo),
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WindowRealmInfo {
    context: BrowsingContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    sandbox: Option<String>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DedicatedWorkerRealmInfo {
    owners: Vec<Realm>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SharedWorkerRealmInfo {
    owners: Vec<Realm>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ServiceWorkerRealmInfo {
    owners: Vec<Realm>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WorkerRealmInfo {}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PaintWorkletRealmInfo {}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AudioWorkletRealmInfo {}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmInfo>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WorkletRealmInfo {}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RealmType>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum RealmType {
    #[serde(rename = "window")]
    Window,
    #[serde(rename = "dedicated-worker")]
    DedicatedWorker,
    #[serde(rename = "shared-worker")]
    SharedWorker,
    #[serde(rename = "service-worker")]
    ServiceWorker,
    #[serde(rename = "worker")]
    Worker,
    #[serde(rename = "paint-worklet")]
    PaintWorklet,
    #[serde(rename = "audio-worklet")]
    AudioWorklet,
    #[serde(rename = "worklet")]
    Worklet,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteReference>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum RemoteReference {
    Shared(SharedReference),
    RemoteObject(RemoteObjectReference),
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteReference>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SharedReference {
    pub shared_id: SharedId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub handle: Option<Handle>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-RemoteReference>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RemoteObjectReference {
    pub handle: Handle,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub shared_id: Option<SharedId>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

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

/// <https://w3c.github.io/webdriver-bidi/#type-script-ResultOwnership>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ResultOwnership {
    Root,
    None,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-SerializationOptions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SerializationOptions {
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    max_dom_depth: Option<u64>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    max_object_depth: Option<u64>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    include_shadow_tree: Option<ShadowTreeType>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-SerializationOptions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ShadowTreeType {
    None,
    Open,
    All,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-SharedId>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SharedId(pub String);

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

/// <https://w3c.github.io/webdriver-bidi/#type-script-StackTrace>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StackTrace {
    pub call_frames: Vec<StackFrame>,
}

/// <https://w3c.github.io/webdriver-bidi/#type-script-Source>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub realm: Realm,
    pub context: Option<BrowsingContext>,
}
