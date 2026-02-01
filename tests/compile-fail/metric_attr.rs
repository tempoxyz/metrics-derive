extern crate metrics;
extern crate metrics_derive;

use metrics::Gauge;
use metrics_derive::Metrics;

fn main() {}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics {
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics2 {
    #[metric()]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics3 {
    #[metric(random = "value")]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics4 {
    #[metric(describe = 123)]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics5 {
    #[metric(rename = 123)]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics6 {
    #[metric(describe = "", describe = "")]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics7 {
    #[metric(rename = "_gauge", rename = "_gauge")]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics8 {
    #[metric(describe = "")]
    gauge: String,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics9 {
    #[metric(describe = "gauge", labels = "not_an_array")]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics10 {
    #[metric(describe = "gauge", labels = ["not_a_tuple"])]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics11 {
    #[metric(describe = "gauge", labels = [("only_one")])]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics12 {
    #[metric(describe = "gauge", labels = [(123, "value")])]
    gauge: Gauge,
}

#[derive(Metrics)]
#[metrics(scope = "some_scope")]
struct CustomMetrics13 {
    #[metric(describe = "gauge", labels = [], labels = [])]
    gauge: Gauge,
}
