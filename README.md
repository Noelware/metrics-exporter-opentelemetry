### 🐻‍❄️🎈 `metrics-exporter-opentelemetry`
#### *A [`metrics`] exporter over OpenTelemetry*

The **metrics-exporter-opentelemetry** crate is a [`metrics`] exporter over OpenTelemetry's **metrics** API.

## Warnings
- The crate doesn't support the following methods from [`metrics`]:
    - [`Counter::absolute`](https://docs.rs/metrics/latest/metrics/struct.Counter.html#method.absolute) - OpenTelemetry doesn't keep track of the value inside of a counter.

    - [`Gauge::increment`](https://docs.rs/metrics/latest/metrics/struct.Gauge.html#method.increment), [`Gauge::decrement`](https://docs.rs/metrics/latest/metrics/struct.Gauge.html#method.decrement) - OpenTelemetry doesn't keep track of the value inside of a gauge.

    - [`Histogram::record_many`](https://docs.rs/metrics/latest/metrics/struct.Histogram.html#method.record_many) - OpenTelemetry doesn't support recording multiple histogram points.

- The crate provide no-op implementations of the `metrics::Recorder::describe_*` as we can't modify a constructed counter/gauge/histogram from `metrics::Recorder::register_*`. The SDK keeps track of it but is internal and isn't able to be accessed.

## Usage
```rust
// Cargo.toml:
//
// [dependencies]
// metrics = "^0"
// metrics-exporter-opentelemetry = "^0"

use metrics_exporter_opentelemetry::Recorder;

fn main() {
    // Install a global `metrics` recorder
    let _ = Recorder::builder("my-app")
        .install_global()
        .unwrap();

    let counter = metrics::counter!("hello.world");
    counter.increment(1);
}
```

## License
**metrics-exporter-opentelemetry** is released under the **MIT License** with love, care, and **Dr. Pepper**. This is a call of help, I am too addicted to Dr. Pepper at this rate.

[`metrics`]: https://crates.io/crates/metrics
