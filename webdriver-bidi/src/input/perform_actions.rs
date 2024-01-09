//! <https://w3c.github.io/webdriver-bidi/#command-input-performActions>

use serde::{Deserialize, Serialize};

use super::ElementOrigin;
use crate::browsing_context::BrowsingContext;
use crate::protocol::EmptyResult;

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename = "input.performActions")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub params: Parameters,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    context: BrowsingContext,
    actions: Vec<SourceActions>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Result(pub EmptyResult);

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum SourceActions {
    #[serde(rename = "none")]
    None(NoneSourceActions),
    #[serde(rename = "key")]
    Key(KeySourceActions),
    #[serde(rename = "pointer")]
    Pointer(PointerSourceActions),
    #[serde(rename = "wheel")]
    Wheel(WheelSourceActions),
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "none")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NoneSourceActions {
    id: String,
    actions: Vec<NoneSourceAction>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NoneSourceAction(pub PauseAction);

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "key")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct KeySourceActions {
    id: String,
    actions: Vec<KeySourceAction>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum KeySourceAction {
    #[serde(rename = "pause")]
    Pause(PauseAction),
    #[serde(rename = "keyDown")]
    KeyDown(KeyDownAction),
    #[serde(rename = "keyUp")]
    KeyUp(KeyUpAction),
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PointerSourceActions {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    parameters: Option<PointerParameters>,
    actions: Vec<PointerSourceAction>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PointerParameters {
    // TODO FIXME default
    pub pointer_type: Option<PointerType>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum PointerSourceAction {
    Pause(PauseAction),
    PointerDown(PointerDownAction),
    PointerUp(PointerUpAction),
    PointerMove(PointerMoveAction),
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "wheel")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WheelSourceActions {
    id: String,
    actions: Vec<WheelSourceAction>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum WheelSourceAction {
    Pause(PauseAction),
    WheelScroll(WheelScrollAction),
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "pause")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PauseAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    duration: Option<u64>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "keyDown")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct KeyDownAction {
    value: String,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "keyUp")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct KeyUpAction {
    value: String,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "pointerUp")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PointerUpAction {
    button: u64,
    #[serde(flatten)]
    common: PointerCommonProperties,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "pointerDown")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PointerDownAction {
    button: u64,
    #[serde(flatten)]
    common: PointerCommonProperties,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "pointerMove")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PointerMoveAction {
    x: i64,
    y: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    origin: Option<Origin>,
    #[serde(flatten)]
    common: PointerCommonProperties,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename = "scroll")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct WheelScrollAction {
    x: i64,
    y: i64,
    delta_x: i64,
    delta_y: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    duration: Option<u64>,
    // TODO FIXME default
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    origin: Option<Origin>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PointerCommonProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub width: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub height: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub pressure: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tangential_pressure: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub twist: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub altitude_angle: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub azimuth_angle: Option<f64>,
}

/// <https://w3c.github.io/webdriver-bidi/#command-input-performActions>
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Origin {
    Viewport,
    Pointer,
    Element(ElementOrigin),
}
