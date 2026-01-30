metrics-derive
===============

[![Build Status](https://github.com/tempoxyz/metrics-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/tempoxyz/metrics-derive/actions)
[![Crates.io](https://img.shields.io/crates/v/metrics-derive.svg)](https://crates.io/crates/metrics-derive)
[![Documentation](https://docs.rs/metrics-derive/badge.svg)](https://docs.rs/metrics-derive)
[![Rust](https://img.shields.io/badge/rust-1.80.0%2B-blue.svg?maxAge=3600)](https://github.com/tempoxyz/metrics-derive)

`metrics-derive` provides the `Metrics` derive macro, allowing you to automatically implement metrics description and initialization.

Originally introduced in [`paradigmxyz/reth`](https://github.com/paradigmxyz/reth) repository in https://github.com/paradigmxyz/reth/pull/592.

## Installation

To use `metrics-derive`, add it to your Cargo.toml:

```toml
[dependencies]
metrics-derive = "0.1"
```

This crate requires a [`metrics`](https://crates.io/crates/metrics) peer dependency.

## Usage

```rust
use metrics::{Counter, Gauge, Histogram};
use metrics_derive::Metrics;

#[derive(Metrics)]
#[metrics(scope = "metrics_custom")]
pub struct CustomMetrics {
    /// A gauge with doc comment description.
    gauge: Gauge,
    #[metric(rename = "second_gauge", describe = "A gauge with metric attribute description.")]
    gauge2: Gauge,
    /// Some doc comment
    #[metric(describe = "Metric attribute description will be preferred over doc comment.")]
    counter: Counter,
    /// A renamed histogram.
    #[metric(rename = "histogram")]
    histo: Histogram,
}
```

The example above will be expanded to:
```rust
pub struct CustomMetrics {
    /// A gauge with doc comment description.
    gauge: metrics::Gauge,
    gauge2: metrics::Gauge,
    /// Some doc comment
    counter: metrics::Counter,
    /// A renamed histogram.
    histo: metrics::Histogram,
}

impl Default for CustomMetrics {
    fn default() -> Self {
        Self {
            gauge: metrics::gauge!("metrics_custom_gauge"),
            gauge2: metrics::gauge!("metrics_custom_second_gauge"),
            counter: metrics::counter!("metrics_custom_counter"),
            histo: metrics::histogram!("metrics_custom_histogram"),
        }
    }
}

impl CustomMetrics {
    /// Describe all exposed metrics
    pub fn describe() {
        metrics::describe_gauge!(
            "metrics_custom_gauge",
            "A gauge with doc comment description."
        );
        metrics::describe_gauge!(
            "metrics_custom_second_gauge",
            "A gauge with metric attribute description."
        );
        metrics::describe_counter!(
            "metrics_custom_counter",
            "Metric attribute description will be preferred over doc comment."
        );
        metrics::describe_histogram!("metrics_custom_histogram", "A renamed histogram.");
    }
}

impl std::fmt::Debug for CustomMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomMetrics").finish()
    }
}
```
