//! This crate provides the [`Metrics`] derive macro for recording [`metrics`].
//!
//! See the [`Metrics`] derive macro for more information.
//!
//! [`metrics`]: https://docs.rs/metrics

#![doc(issue_tracker_base_url = "https://github.com/tempoxyz/metrics-derive/issues/")]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
mod metric;
mod with_attrs;

/// Instruments all of the struct fields and creates a [`Default`] implementation for the struct
/// recording all of the metrics.
///
/// Additionally, it creates a `describe` method on the struct, which
/// internally calls the describe statements for all metric fields.
///
/// Sample usage:
/// ```
/// use metrics::{Counter, Gauge, Histogram};
/// use metrics_derive::Metrics;
///
/// #[derive(Metrics)]
/// #[metrics(scope = "metrics_custom")]
/// pub struct CustomMetrics {
///     /// A gauge with doc comment description.
///     gauge: Gauge,
///     #[metric(rename = "second_gauge", describe = "A gauge with metric attribute description.")]
///     gauge2: Gauge,
///     /// Some doc comment
///     #[metric(describe = "Metric attribute description will be preferred over doc comment.")]
///     counter: Counter,
///     /// A renamed histogram.
///     #[metric(rename = "histogram")]
///     histo: Histogram,
/// }
/// ```
///
/// The example above will be expanded to:
/// ```
/// pub struct CustomMetrics {
///     /// A gauge with doc comment description.
///     gauge: metrics::Gauge,
///     gauge2: metrics::Gauge,
///     /// Some doc comment
///     counter: metrics::Counter,
///     /// A renamed histogram.
///     histo: metrics::Histogram,
/// }
///
/// impl Default for CustomMetrics {
///     fn default() -> Self {
///         Self {
///             gauge: metrics::gauge!("metrics_custom_gauge"),
///             gauge2: metrics::gauge!("metrics_custom_second_gauge"),
///             counter: metrics::counter!("metrics_custom_counter"),
///             histo: metrics::histogram!("metrics_custom_histogram"),
///         }
///     }
/// }
///
/// impl CustomMetrics {
///     /// Describe all exposed metrics
///     pub fn describe() {
///         metrics::describe_gauge!(
///             "metrics_custom_gauge",
///             "A gauge with doc comment description."
///         );
///         metrics::describe_gauge!(
///             "metrics_custom_second_gauge",
///             "A gauge with metric attribute description."
///         );
///         metrics::describe_counter!(
///             "metrics_custom_counter",
///             "Metric attribute description will be preferred over doc comment."
///         );
///         metrics::describe_histogram!("metrics_custom_histogram", "A renamed histogram.");
///     }
/// }
///
/// impl std::fmt::Debug for CustomMetrics {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         f.debug_struct("CustomMetrics").finish()
///     }
/// }
/// ```
///
/// Similarly, you can derive metrics with "dynamic" scope,
/// meaning their scope can be set at the time of instantiation.
/// For example:
/// ```
/// use metrics_derive::Metrics;
///
/// #[derive(Metrics)]
/// #[metrics(dynamic = true)]
/// pub struct DynamicScopeMetrics {
///     /// A gauge with doc comment description.
///     gauge: metrics::Gauge,
/// }
/// ```
///
/// The example with dynamic scope will expand to
/// ```
/// pub struct DynamicScopeMetrics {
///     /// A gauge with doc comment description.
///     gauge: metrics::Gauge,
/// }
///
/// impl DynamicScopeMetrics {
///     pub fn new(scope: &str) -> Self {
///         Self { gauge: metrics::gauge!(format!("{}{}{}", scope, "_", "gauge")) }
///     }
///
///     pub fn describe(scope: &str) {
///         metrics::describe_gauge!(
///             format!("{}{}{}", scope, "_", "gauge"),
///             "A gauge with doc comment description."
///         );
///     }
/// }
///
/// impl std::fmt::Debug for DynamicScopeMetrics {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         f.debug_struct("DynamicScopeMetrics").finish()
///     }
/// }
/// ```
///
/// ## Per-field labels
///
/// You can attach labels to individual fields using the `labels` attribute.
/// This is useful when you want multiple fields to share the same metric name
/// but be distinguished by label values:
///
/// ```
/// use metrics::Counter;
/// use metrics_derive::Metrics;
///
/// #[derive(Metrics)]
/// #[metrics(scope = "forwarder")]
/// pub struct TransactionMetrics {
///     /// Number of transactions.
///     #[metric(rename = "transactions", labels = [("outcome", "forwarded")])]
///     forwarded: Counter,
///     /// Number of transactions.
///     #[metric(rename = "transactions", labels = [("outcome", "dropped")])]
///     dropped: Counter,
/// }
/// ```
///
/// Field-level labels are appended to any struct-level labels passed via
/// `new_with_labels()`. Multiple labels can be specified per field:
///
/// ```
/// use metrics::Counter;
/// use metrics_derive::Metrics;
///
/// #[derive(Metrics)]
/// #[metrics(scope = "api")]
/// pub struct RequestMetrics {
///     /// Request count.
///     #[metric(rename = "requests", labels = [("method", "GET"), ("status", "200")])]
///     get_success: Counter,
/// }
/// ```
///
/// ## Global labels
///
/// You can also specify labels at the struct level that apply to all fields.
/// These are combined with any instance labels passed via `new_with_labels()`:
///
/// ```
/// use metrics::Counter;
/// use metrics_derive::Metrics;
///
/// #[derive(Metrics)]
/// #[metrics(scope = "api", labels = [("service", "gateway"), ("version", "v1")])]
/// pub struct ApiMetrics {
///     /// Total requests.
///     requests: Counter,
///     /// Successful requests (with additional field-level labels).
///     #[metric(labels = [("status", "success")])]
///     success: Counter,
/// }
/// ```
///
/// Label order: instance labels (from `new_with_labels`) → global labels → field labels.
///
/// Label values can also be arbitrary expressions, such as constants:
///
/// ```
/// use metrics::Counter;
/// use metrics_derive::Metrics;
///
/// const SERVICE_NAME: &str = "gateway";
///
/// #[derive(Metrics)]
/// #[metrics(scope = "api", labels = [("service", SERVICE_NAME)])]
/// pub struct ConstLabelMetrics {
///     /// Total requests.
///     requests: Counter,
/// }
/// ```
#[proc_macro_derive(Metrics, attributes(metrics, metric))]
pub fn derive_metrics(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input).unwrap_or_else(|err| err.to_compile_error()).into()
}
