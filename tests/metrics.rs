// Use `cargo nextest` to run tests.

#![allow(missing_docs)]
use metrics::{
    Counter, Gauge, Histogram, Key, KeyName, Label, Metadata, Recorder, SharedString, Unit,
};
use metrics_derive::Metrics;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

#[allow(dead_code)]
#[derive(Clone, Metrics)]
#[metrics(scope = "metrics_custom")]
pub struct CustomMetrics {
    #[metric(skip)]
    skipped_field_a: u8,
    /// A gauge with doc comment description.
    gauge: Gauge,
    #[metric(rename = "second_gauge", describe = "A gauge with metric attribute description.")]
    gauge2: Gauge,
    #[metric(skip)]
    skipped_field_b: u16,
    /// Some doc comment
    #[metric(describe = "Metric attribute description will be preferred over doc comment.")]
    counter: Counter,
    #[metric(skip)]
    skipped_field_c: u32,
    #[metric(skip)]
    skipped_field_d: u64,
    /// A renamed histogram.
    #[metric(rename = "histogram")]
    histo: Histogram,
    #[metric(skip)]
    skipped_field_e: u128,
}

impl CustomMetrics {
    /// Custom new fn.
    pub fn new() -> Self {
        Self::new_with_labels(&[("type", "custom")])
    }
}

#[allow(dead_code)]
#[derive(Metrics)]
#[metrics(dynamic = true)]
struct DynamicScopeMetrics {
    #[metric(skip)]
    skipped_field_a: u8,
    /// A gauge with doc comment description.
    gauge: Gauge,
    #[metric(rename = "second_gauge", describe = "A gauge with metric attribute description.")]
    gauge2: Gauge,
    #[metric(skip)]
    skipped_field_b: u16,
    /// Some doc comment
    #[metric(describe = "Metric attribute description will be preferred over doc comment.")]
    counter: Counter,
    #[metric(skip)]
    skipped_field_c: u32,
    #[metric(skip)]
    skipped_field_d: u64,
    /// A renamed histogram.
    #[metric(rename = "histogram")]
    histo: Histogram,
    #[metric(skip)]
    skipped_field_e: u128,
}

/// Tests per-field labels with static scope.
#[allow(dead_code)]
#[derive(Metrics)]
#[metrics(scope = "transactions")]
struct FieldLabelMetrics {
    /// Number of transactions.
    #[metric(rename = "count", labels = [("outcome", "forwarded")])]
    forwarded: Counter,
    /// Number of transactions.
    #[metric(rename = "count", labels = [("outcome", "dropped")])]
    dropped: Counter,
    /// Number of transactions.
    #[metric(rename = "count", labels = [("outcome", "processed"), ("priority", "high")])]
    processed_high: Counter,
}

/// Tests per-field labels with dynamic scope.
#[allow(dead_code)]
#[derive(Metrics)]
#[metrics(dynamic = true)]
struct DynamicFieldLabelMetrics {
    /// Number of transactions.
    #[metric(rename = "count", labels = [("outcome", "forwarded")])]
    forwarded: Counter,
    /// Number of transactions.
    #[metric(rename = "count", labels = [("outcome", "dropped")])]
    dropped: Counter,
}

static RECORDER: LazyLock<TestRecorder> = LazyLock::new(TestRecorder::new);

fn test_describe(scope: &str) {
    assert_eq!(RECORDER.metrics_len(), 4);

    let gauge = RECORDER.get_metric(&format!("{scope}.gauge"));
    assert!(gauge.is_some());
    assert_eq!(
        gauge.unwrap(),
        TestMetric {
            ty: TestMetricTy::Gauge,
            description: Some("A gauge with doc comment description.".to_owned()),
            labels: None,
        }
    );

    let second_gauge = RECORDER.get_metric(&format!("{scope}.second_gauge"));
    assert!(second_gauge.is_some());
    assert_eq!(
        second_gauge.unwrap(),
        TestMetric {
            ty: TestMetricTy::Gauge,
            description: Some("A gauge with metric attribute description.".to_owned()),
            labels: None,
        }
    );

    let counter = RECORDER.get_metric(&format!("{scope}.counter"));
    assert!(counter.is_some());
    assert_eq!(
        counter.unwrap(),
        TestMetric {
            ty: TestMetricTy::Counter,
            description: Some(
                "Metric attribute description will be preferred over doc comment.".to_owned()
            ),
            labels: None,
        }
    );

    let histogram = RECORDER.get_metric(&format!("{scope}.histogram"));
    assert!(histogram.is_some());
    assert_eq!(
        histogram.unwrap(),
        TestMetric {
            ty: TestMetricTy::Histogram,
            description: Some("A renamed histogram.".to_owned()),
            labels: None,
        }
    );
}

#[test]
fn describe_metrics() {
    let _guard = RECORDER.enter();

    CustomMetrics::describe();

    test_describe("metrics_custom");
}

#[test]
fn describe_dynamic_metrics() {
    let _guard = RECORDER.enter();

    let scope = "local_scope";

    DynamicScopeMetrics::describe(scope);

    test_describe(scope);
}

fn test_register(scope: &str) {
    assert_eq!(RECORDER.metrics_len(), 4);

    let gauge = RECORDER.get_metric(&format!("{scope}.gauge"));
    assert!(gauge.is_some());
    assert_eq!(
        gauge.unwrap(),
        TestMetric { ty: TestMetricTy::Gauge, description: None, labels: None }
    );

    let second_gauge = RECORDER.get_metric(&format!("{scope}.second_gauge"));
    assert!(second_gauge.is_some());
    assert_eq!(
        second_gauge.unwrap(),
        TestMetric { ty: TestMetricTy::Gauge, description: None, labels: None }
    );

    let counter = RECORDER.get_metric(&format!("{scope}.counter"));
    assert!(counter.is_some());
    assert_eq!(
        counter.unwrap(),
        TestMetric { ty: TestMetricTy::Counter, description: None, labels: None }
    );

    let histogram = RECORDER.get_metric(&format!("{scope}.histogram"));
    assert!(histogram.is_some());
    assert_eq!(
        histogram.unwrap(),
        TestMetric { ty: TestMetricTy::Histogram, description: None, labels: None }
    );
}

#[test]
fn register_metrics() {
    let _guard = RECORDER.enter();

    let _metrics = CustomMetrics::default();

    test_register("metrics_custom");
}

#[test]
fn register_dynamic_metrics() {
    let _guard = RECORDER.enter();

    let scope = "local_scope";

    let _metrics = DynamicScopeMetrics::new(scope);

    test_register(scope);
}

fn test_labels(scope: &str) {
    let test_labels = vec![Label::new("key", "value")];

    let gauge = RECORDER.get_metric(&format!("{scope}.gauge"));
    assert!(gauge.is_some());
    let labels = gauge.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels,);

    let second_gauge = RECORDER.get_metric(&format!("{scope}.second_gauge"));
    assert!(second_gauge.is_some());
    let labels = second_gauge.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels,);

    let counter = RECORDER.get_metric(&format!("{scope}.counter"));
    assert!(counter.is_some());
    let labels = counter.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels,);

    let histogram = RECORDER.get_metric(&format!("{scope}.histogram"));
    assert!(histogram.is_some());
    let labels = histogram.unwrap().labels;
    assert!(labels.is_some());
    assert_eq!(labels.unwrap(), test_labels,);
}

#[test]
fn label_metrics() {
    let _guard = RECORDER.enter();

    let _metrics = CustomMetrics::new_with_labels(&[("key", "value")]);

    test_labels("metrics_custom");
}

#[test]
fn dynamic_label_metrics() {
    let _guard = RECORDER.enter();

    let scope = "local_scope";

    let _metrics = DynamicScopeMetrics::new_with_labels(scope, &[("key", "value")]);

    test_labels(scope);
}

#[test]
fn field_labels_static() {
    let _guard = RECORDER.enter();

    let _metrics = FieldLabelMetrics::new_with_labels(&[("env", "prod")]);

    // "forwarded" field: struct labels + field labels
    let forwarded = RECORDER.get_metric("transactions.count");
    assert!(forwarded.is_some());
    let metric = forwarded.unwrap();
    assert_eq!(metric.ty, TestMetricTy::Counter);
    // We can't distinguish between the three "count" metrics by name alone,
    // but we can check that at least one was registered with the expected labels.
    // The last one registered will be stored (processed_high with 3 labels).
    let labels = metric.labels.unwrap();
    // Last registered is processed_high: env=prod, outcome=processed, priority=high
    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&Label::new("env", "prod")));
    assert!(labels.contains(&Label::new("outcome", "processed")));
    assert!(labels.contains(&Label::new("priority", "high")));
}

#[test]
fn field_labels_dynamic() {
    let _guard = RECORDER.enter();

    let scope = "dynamic_tx";

    let _metrics = DynamicFieldLabelMetrics::new_with_labels(scope, &[("env", "staging")]);

    // Check that field labels are appended to struct labels
    let metric = RECORDER.get_metric(&format!("{scope}.count"));
    assert!(metric.is_some());
    let metric = metric.unwrap();
    assert_eq!(metric.ty, TestMetricTy::Counter);
    // Last registered is "dropped": env=staging, outcome=dropped
    let labels = metric.labels.unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&Label::new("env", "staging")));
    assert!(labels.contains(&Label::new("outcome", "dropped")));
}

struct TestRecorder {
    // Metrics map: key => Option<description>
    metrics: Mutex<HashMap<String, TestMetric>>,
}

#[derive(PartialEq, Clone, Debug)]
enum TestMetricTy {
    Counter,
    Gauge,
    Histogram,
}

#[derive(PartialEq, Clone, Debug)]
struct TestMetric {
    ty: TestMetricTy,
    description: Option<String>,
    labels: Option<Vec<Label>>,
}

impl TestRecorder {
    fn new() -> Self {
        Self { metrics: Mutex::new(HashMap::default()) }
    }

    /// Sets this recorder as the global recorder for the duration of the returned guard.
    #[must_use]
    fn enter(&'static self) -> impl Drop {
        struct Reset {
            recorder: &'static TestRecorder,
        }
        impl Drop for Reset {
            fn drop(&mut self) {
                self.recorder.clear();
            }
        }

        let _ = metrics::set_global_recorder(self);
        Reset { recorder: self }
    }

    fn metrics_len(&self) -> usize {
        self.metrics.lock().expect("failed to lock metrics").len()
    }

    fn get_metric(&self, key: &str) -> Option<TestMetric> {
        self.metrics.lock().expect("failed to lock metrics").get(key).cloned()
    }

    fn record_metric(
        &self,
        key: &str,
        ty: TestMetricTy,
        description: Option<String>,
        labels: Option<Vec<Label>>,
    ) {
        self.metrics
            .lock()
            .expect("failed to lock metrics")
            .insert(key.to_owned(), TestMetric { ty, description, labels });
    }

    fn clear(&self) {
        self.metrics.lock().expect("failed to lock metrics").clear();
    }
}

impl Recorder for &'static TestRecorder {
    fn describe_counter(&self, key: KeyName, _unit: Option<Unit>, description: SharedString) {
        self.record_metric(
            key.as_str(),
            TestMetricTy::Counter,
            Some(description.into_owned()),
            None,
        )
    }

    fn describe_gauge(&self, key: KeyName, _unit: Option<Unit>, description: SharedString) {
        self.record_metric(key.as_str(), TestMetricTy::Gauge, Some(description.into_owned()), None)
    }

    fn describe_histogram(&self, key: KeyName, _unit: Option<Unit>, description: SharedString) {
        self.record_metric(
            key.as_str(),
            TestMetricTy::Histogram,
            Some(description.into_owned()),
            None,
        )
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        let labels_vec: Vec<Label> = key.labels().cloned().collect();
        let labels = (!labels_vec.is_empty()).then_some(labels_vec);
        self.record_metric(key.name(), TestMetricTy::Counter, None, labels);
        Counter::noop()
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        let labels_vec: Vec<Label> = key.labels().cloned().collect();
        let labels = (!labels_vec.is_empty()).then_some(labels_vec);
        self.record_metric(key.name(), TestMetricTy::Gauge, None, labels);
        Gauge::noop()
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        let labels_vec: Vec<Label> = key.labels().cloned().collect();
        let labels = (!labels_vec.is_empty()).then_some(labels_vec);
        self.record_metric(key.name(), TestMetricTy::Histogram, None, labels);
        Histogram::noop()
    }
}
