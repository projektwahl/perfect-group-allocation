#![feature(error_generic_member_access)]
#![feature(lint_reasons)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

//! Client for the [WebDriver BiDi protocol](https://w3c.github.io/webdriver-bidi/).
//!
//! <div class="warning">This crate is in a very early stage of development! Expect frequent breaking changes.</div>
//!
//! The currently implemented version is Editor’s Draft, 5 January 2024.
//!
//! ## Implementation Notes
//!
//! The types in <https://w3c.github.io/webdriver-bidi/> are converted with the following in mind:
//!
//! The specification uses the [Concise Data Definition Language (CDDL)](https://www.rfc-editor.org/rfc/rfc8610).
//!
//! They are converted as much as possible as-is to make it easy to update them.
//!
//! They are converted with performance and ease-of-use in mind.
//! For example tagged unions are converted in a way that only one parsing attempt is needed (so no untagged).
//!
//! ### Type conversion rules
//! | Spec type    | Rust type                           |
//! |--------------|-------------------------------------|
//! | `js-uint`    | [`u64`]                             |
//! | `js-int`     | [`i64`]                             |
//!
//! ### Serde rules
//! All types will (at some point) be annotated with:
//! ```
//! #[serde(rename_all = "camelCase")]
//! #[serde(deny_unknown_fields)]
//! ```
//!
//! To specify a fallback type on a tagged enum **variant**, use:
//! ```
//! #[serde(untagged)]
//! ```
//!
//! All fields in the spec starting with `? ` are optional and are represented as `Option<T>` and annotated with:
//! ```rust
//! #[serde(skip_serializing_if = "Option::is_none")]
//! #[serde(default)]
//! ```
//!
//! `Extensible`` type is represented as:
//! ```
//! #
//! #[serde(flatten)]
//! pub extensible: protocol::Extensible
//! ```
//!
//! Otherwise `Type / null` is represented with an `Option<Type>`.
//!
//! Types combined in the spec with `( A // B // ... )` are represented as enum though usually as a tagged enum for performance.

pub mod browsing_context;
mod generated;
pub mod log;
pub mod protocol;
mod result;
pub mod script;
pub mod session;
mod webdriver;
mod webdriver_handler;
mod webdriver_session;

use browsing_context::BrowsingContext;
use generated::EventData;
pub use result::{Error, ErrorInner, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use webdriver::WebDriver;

pub trait ExtractBrowsingContext {
    fn browsing_context(&self) -> Option<&BrowsingContext>;
}
