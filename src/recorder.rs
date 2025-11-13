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
    global, metrics::{Meter, MeterProvider}, InstrumentationScope, InstrumentationScopeBuilder,
    KeyValue,
};
use opentelemetry_sdk::metrics::{MeterProviderBuilder, SdkMeterProvider};
use std::{
    borrow::Cow,
    collections::HashMap,
    ops::Deref,
    sync::{
        atomic::{AtomicU64, Ordering}, Arc,
        Mutex,
    },
};

/// A builder for constructing a [`Recorder`].
#[derive(Debug)]
pub struct Builder {
    builder: MeterProviderBuilder,
    scope: InstrumentationScopeBuilder,
}

impl Builder {
    /// Runs the closure (`f`) to modify the [`MeterProviderBuilder`] to build a
    /// [`MeterProvider`](MeterProvider).
    pub fn with_meter_provider(mut self, f: impl FnOnce(MeterProviderBuilder) -> MeterProviderBuilder) -> Self {
        self.builder = f(self.builder);
        self
    }

    /// Modify the [`InstrumentationScope`] to provide additional metadata from the
    /// closure (`f`).
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
    /// provider to [`global`], use [`Builder::install_global`].
    pub fn build(self) -> (SdkMeterProvider, Recorder) {
        let provider = self.builder.build();
        let meter = provider.meter_with_scope(self.scope.build());

        (provider, Recorder {
            meter,
            metrics_metadata: Arc::new(Mutex::new(HashMap::new())),
        })
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

#[derive(Debug)]
struct MetricMetadata {
    unit: Option<Unit>,
    description: SharedString,
}

/// A standard recorder that implements [`metrics::Recorder`].
///
/// This instance implements <code>[`Deref`]\<Target = [`Meter`]\></code>, so
/// you can still interact with the SDK's initialized [`Meter`] instance.
#[derive(Debug, Clone)]
pub struct Recorder {
    meter: Meter,
    metrics_metadata: Arc<Mutex<HashMap<KeyName, MetricMetadata>>>,
}

impl Recorder {
    /// Creates a new [`Builder`] with a given name for instrumentation.
    pub fn builder<S: Into<Cow<'static, str>>>(name: S) -> Builder {
        Builder {
            builder: MeterProviderBuilder::default(),
            scope: InstrumentationScope::builder(name.into()),
        }
    }

    /// Creates a [`Recorder`] with an already established [`Meter`].
    pub fn with_meter(meter: Meter) -> Self {
        Recorder {
            meter,
            metrics_metadata: Arc::new(Mutex::new(HashMap::new())),
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
    fn describe_counter(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        let mut metrics_metadata = self.metrics_metadata.lock().unwrap();
        metrics_metadata.insert(key, MetricMetadata { unit, description });
    }

    fn describe_gauge(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        let mut metrics_metadata = self.metrics_metadata.lock().unwrap();
        metrics_metadata.insert(key, MetricMetadata { unit, description });
    }

    fn describe_histogram(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        let mut metrics_metadata = self.metrics_metadata.lock().unwrap();
        metrics_metadata.insert(key, MetricMetadata { unit, description });
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        let mut builder = self.meter.u64_counter(key.name().to_owned());
        if let Some(metadata) = self.metrics_metadata.lock().unwrap().remove(key.name()) {
            if let Some(unit) = metadata.unit {
                builder = builder.with_unit(unit.as_canonical_label());
            }
            builder = builder.with_description(metadata.description.to_string());
        }

        let counter = builder.build();
        let labels = key
            .labels()
            .map(|label| KeyValue::new(label.key().to_owned(), label.value().to_owned()))
            .collect();

        Counter::from_arc(Arc::new(WrappedCounter {
            counter,
            labels,
            value: AtomicU64::new(0),
        }))
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        let mut builder = self.meter.f64_gauge(key.name().to_owned());
        if let Some(metadata) = self.metrics_metadata.lock().unwrap().remove(key.name()) {
            if let Some(unit) = metadata.unit {
                builder = builder.with_unit(unit.as_canonical_label());
            }
            builder = builder.with_description(metadata.description.to_string());
        }

        let gauge = builder.build();
        let labels = key
            .labels()
            .map(|label| KeyValue::new(label.key().to_owned(), label.value().to_owned()))
            .collect();

        Gauge::from_arc(Arc::new(WrappedGauge {
            gauge,
            labels,
            value: AtomicU64::new(0),
        }))
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        let mut builder = self.meter.f64_histogram(key.name().to_owned());
        if let Some(metadata) = self.metrics_metadata.lock().unwrap().remove(key.name()) {
            if let Some(unit) = metadata.unit {
                builder = builder.with_unit(unit.as_canonical_label());
            }
            builder = builder.with_description(metadata.description.to_string());
        }

        let histogram = builder.build();
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
    value: AtomicU64,
}

impl CounterFn for WrappedCounter {
    fn increment(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
        self.counter.add(value, &self.labels);
    }

    fn absolute(&self, value: u64) {
        let prev = self.value.swap(value, Ordering::Relaxed);
        let diff = value.saturating_sub(prev);
        self.counter.add(diff, &self.labels);
    }
}

struct WrappedGauge {
    gauge: opentelemetry::metrics::Gauge<f64>,
    labels: Vec<KeyValue>,
    value: AtomicU64,
}

impl GaugeFn for WrappedGauge {
    fn increment(&self, value: f64) {
        let mut current = self.value.load(Ordering::Relaxed);
        let mut new = f64::from_bits(current) + value;
        while let Err(val) = self
            .value
            .compare_exchange(current, new.to_bits(), Ordering::AcqRel, Ordering::Relaxed)
        {
            current = val;
            new = f64::from_bits(current) + value;
        }

        self.gauge.record(new, &self.labels);
    }

    fn decrement(&self, value: f64) {
        let mut current = self.value.load(Ordering::Relaxed);
        let mut new = f64::from_bits(current) - value;
        while let Err(val) = self
            .value
            .compare_exchange(current, new.to_bits(), Ordering::AcqRel, Ordering::Relaxed)
        {
            current = val;
            new = f64::from_bits(current) - value;
        }

        self.gauge.record(new, &self.labels);
    }

    fn set(&self, value: f64) {
        self.value.store(value.to_bits(), Ordering::Relaxed);
        self.gauge.record(value, &self.labels);
    }
}

struct WrappedHistogram {
    histogram: opentelemetry::metrics::Histogram<f64>,
    labels: Vec<KeyValue>,
}

impl HistogramFn for WrappedHistogram {
    fn record(&self, value: f64) {
        self.histogram.record(value, &self.labels);
    }

    fn record_many(&self, value: f64, count: usize) {
        for _ in 0..count {
            self.histogram.record(value, &self.labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_sdk::metrics::Temporality;

    #[test]
    fn standard_usage() {
        let exporter = opentelemetry_stdout::MetricExporterBuilder::default()
            .with_temporality(Temporality::Cumulative)
            .build();

        let (provider, recorder) = Recorder::builder("my-app")
            .with_meter_provider(|builder| builder.with_periodic_exporter(exporter))
            .build();

        global::set_meter_provider(provider.clone());
        metrics::set_global_recorder(recorder).unwrap();

        let counter = metrics::counter!("my-counter");
        counter.increment(1);

        provider.force_flush().unwrap();
    }
}
