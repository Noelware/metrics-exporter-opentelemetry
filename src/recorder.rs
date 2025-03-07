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

use metrics::{Counter, CounterFn, Gauge, GaugeFn, Histogram, HistogramFn, Key, KeyName, Metadata, SharedString, Unit};
use opentelemetry::{
    global,
    metrics::{Meter, MeterProvider},
    InstrumentationScope, InstrumentationScopeBuilder, KeyValue,
};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, SdkMeterProvider};
use std::{borrow::Cow, ops::Deref, sync::Arc};

/// A builder for constructing a [`Recorder`].
#[derive(Debug)]
pub struct Builder {
    builder: MeterProviderBuilder,
    scope: InstrumentationScopeBuilder,
}

impl Builder {
    /// Runs the closure (`f`) to modify the [`MeterProviderBuilder`] to build a
    /// [`MeterProvider`](opentelemetry::metrics::MeterProvider).
    ///
    /// ## Example
    /// ```
    /// # use metrics_exporter_opentelemetry::Recorder;
    /// #
    /// let recorder = Recorder::builder("my-app")
    ///     .with_meter_provider(|builder| {
    ///         // modify `builder` to your liking
    ///     })
    ///     .build();
    /// ```
    pub fn with_meter_provider(mut self, f: impl FnOnce(MeterProviderBuilder) -> MeterProviderBuilder) -> Self {
        self.builder = f(self.builder);
        self
    }

    /// Modify the [`InstrumentationScope`] to provide additional metadata from the
    /// closure (`f`).
    ///
    /// ## Example
    /// ```
    /// # use metrics_exporter_opentelemetry::Recorder;
    /// #
    /// let (_, recorder) = Recorder::builder("my-app")
    ///     .with_instrumentation_scope(|builder| {
    ///         // modify `builder` to your liking
    ///     })
    ///     .build();
    /// ```
    pub fn with_instrumentation_scope(
        mut self,
        f: impl FnOnce(InstrumentationScopeBuilder) -> InstrumentationScopeBuilder,
    ) -> Self {
        self.scope = f(self.scope);
        self
    }

    /// Consumes the builder and builds a new [`Recorder`] and returns
    /// a [`SdkMeterProvider`].
    ///
    /// A [`SdkMeterProvider`] is provided so you have the responsibility to
    /// do whatever you need to do with it.
    ///
    /// This will not install the recorder as the global recorder for
    /// the [`metrics`] crate, use [`Builder::install`]. This will not install a meter
    /// provider to [`opentelemetry::global`], use [`Builder::install_global`].
    pub fn build(self) -> (SdkMeterProvider, Recorder) {
        let provider = self.builder.build();
        let meter = provider.meter_with_scope(self.scope.build());

        (provider, Recorder { meter })
    }

    /// Builds a [`Recorder`] and sets it as the global recorder for the [`metrics`]
    /// crate.
    ///
    /// This method will not call [`global::set_meter_provider`] for OpenTelemetry and
    /// will be returned as the first element in the return's type tuple.
    pub fn install(self) -> crate::Result<(SdkMeterProvider, Recorder)> {
        let (provider, recorder) = self.build();
        metrics::set_global_recorder(recorder.clone())?;

        Ok((provider, recorder))
    }

    /// Builds the [`Recorder`] to record metrics to OpenTelemetry, set the global
    /// recorder for the [`metrics`] crate, and calls [`global::set_meter_provider`]
    /// to set the constructed [`SdkMeterProvider`].
    pub fn install_global(self) -> crate::Result<Recorder> {
        let (provider, recorder) = self.install()?;
        global::set_meter_provider(provider);

        Ok(recorder)
    }
}

/// A standard recorder that implements [`metrics::Recorder`].
///
/// This instance implements <code>[`Deref`]\<Target = [`Meter`]\></code>, so
/// you can still interact with the SDK's initialized [`Meter`] instance.
#[derive(Debug, Clone)]
pub struct Recorder {
    meter: Meter,
}

impl Recorder {
    /// Creates a new [`Builder`] with a given name for instrumentation.
    pub fn builder<S: Into<Cow<'static, str>>>(name: S) -> Builder {
        Builder {
            builder: MeterProviderBuilder::default(),
            scope: InstrumentationScope::builder(name.into()),
        }
    }
}

impl Deref for Recorder {
    type Target = Meter;

    fn deref(&self) -> &Self::Target {
        &self.meter
    }
}

impl metrics::Recorder for Recorder {
    fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // TODO(@auguwu): is there any way we can support this?
    }

    fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // TODO(@auguwu): is there any way we can support this?
    }

    fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {
        // TODO(@auguwu): is there any way we can support this?
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        let counter = self.meter.u64_counter(key.name().to_owned()).build();
        let labels = key
            .labels()
            .map(|label| KeyValue::new(label.key().to_owned(), label.value().to_owned()))
            .collect();

        Counter::from_arc(Arc::new(WrappedCounter { counter, labels }))
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        let gauge = self.meter.f64_gauge(key.name().to_owned()).build();
        let labels = key
            .labels()
            .map(|label| KeyValue::new(label.key().to_owned(), label.value().to_owned()))
            .collect();

        Gauge::from_arc(Arc::new(WrappedGauge { gauge, labels }))
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        let histogram = self.meter.f64_histogram(key.name().to_owned()).build();
        let labels = key
            .labels()
            .map(|label| KeyValue::new(label.key().to_owned(), label.value().to_owned()))
            .collect();

        Histogram::from_arc(Arc::new(WrappedHistogram { histogram, labels }))
    }
}

struct WrappedCounter {
    counter: opentelemetry::metrics::Counter<u64>,
    labels: Vec<KeyValue>,
}

impl CounterFn for WrappedCounter {
    fn increment(&self, value: u64) {
        self.counter.add(value, &self.labels);
    }

    fn absolute(&self, _value: u64) {}
}

struct WrappedGauge {
    gauge: opentelemetry::metrics::Gauge<f64>,
    labels: Vec<KeyValue>,
}

impl GaugeFn for WrappedGauge {
    fn set(&self, value: f64) {
        self.gauge.record(value, &self.labels);
    }

    fn decrement(&self, _value: f64) {}

    fn increment(&self, _value: f64) {}
}

struct WrappedHistogram {
    histogram: opentelemetry::metrics::Histogram<f64>,
    labels: Vec<KeyValue>,
}

impl HistogramFn for WrappedHistogram {
    fn record(&self, value: f64) {
        self.histogram.record(value, &self.labels);
    }

    fn record_many(&self, _value: f64, _count: usize) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_sdk::metrics::Temporality;

    #[test]
    fn standard_usage() {
        let (provider, recorder) = Recorder::builder("my-app")
            .with_meter_provider(|builder| {
                let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
                    .with_temporality(Temporality::Delta)
                    .build();
                builder.with_periodic_exporter(exporter)
            })
            .build();

        global::set_meter_provider(provider);
        metrics::set_global_recorder(recorder).unwrap();

        let counter = metrics::counter!("my-counter");
        counter.increment(1);
    }
}
