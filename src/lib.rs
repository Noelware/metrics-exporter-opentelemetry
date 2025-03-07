// 🐻‍❄️🎈 metrics-exporter-opentelemetry: metrics exporter over OpenTelemetry
// Copyright (c) 2025 Noelware, LLC. <team@noelware.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! # 🐻‍❄️🎈 `metrics-exporter-opentelemetry`
//!
//! The **metrics-exporter-opentelemetry** crate is a [`metrics`] exporter over
//! OpenTelemetry's **metrics** API.
//!
//! ## Warnings
//! - The crate doesn't support the following methods from [`metrics`]:
//!     - [`Counter::absolute`](https://docs.rs/metrics/latest/metrics/struct.Counter.html#method.absolute):
//!       OpenTelemetry doesn't keep track of the value inside of a counter.
//!
//!     - [`Gauge::increment`](https://docs.rs/metrics/latest/metrics/struct.Gauge.html#method.increment),
//!       [`Gauge::decrement`](https://docs.rs/metrics/latest/metrics/struct.Gauge.html#method.decrement):
//!       OpenTelemetry doesn't keep track of the value inside of a gauge.
//!
//!     - [`Histogram::record_many`](https://docs.rs/metrics/latest/metrics/struct.Histogram.html#method.record_many):
//!       OpenTelemetry doesn't support recording multiple histogram points.
//!
//! - The crate provide no-op implementations of the `metrics::Recorder::describe_*` as we
//!   can't modify a constructed counter/gauge/histogram from
//!   `metrics::Recorder::register_*`. The SDK keeps track of it but is internal and isn't
//!   able to be accessed.
//!
//! ## Usage
//! ```rust
//! // Cargo.toml:
//! //
//! // [dependencies]
//! // metrics = "^0"
//! // metrics-exporter-opentelemetry = "^0"
//!
//! use metrics_exporter_opentelemetry::Recorder;
//!
//! # fn main() {
//! // Install a global `metrics` recorder
//! let _ = Recorder::builder("my-app")
//!     .install_global()
//!     .unwrap();
//!
//! let counter = metrics::counter!("hello.world");
//! counter.increment(1);
//! # }
//! ```
//!
//! [`metrics`]: https://crates.io/crates/metrics

#![cfg_attr(any(noeldoc, docsrs), feature(doc_cfg))]
#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![doc(html_favicon_url = "https://cdn.floofy.dev/images/trans.png")]

mod error;
mod recorder;

pub use error::*;
pub use recorder::*;
