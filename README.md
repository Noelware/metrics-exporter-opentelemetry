### 🐻‍❄️🎈 `metrics-exporter-opentelemetry`
#### *A [`metrics`] exporter over OpenTelemetry*

The **metrics-exporter-opentelemetry** crate is a [`metrics`] exporter over OpenTelemetry's **metrics** API.

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
