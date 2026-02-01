extern crate metrics_derive;
use metrics_derive::Metrics;

fn main() {}

#[derive(Metrics)]
struct CustomMetrics;

#[derive(Metrics)]
#[metrics()]
#[metrics()]
struct CustomMetrics2;

#[derive(Metrics)]
#[metrics()]
struct CustomMetrics3;

#[derive(Metrics)]
#[metrics(scope = value)]
struct CustomMetrics4;

#[derive(Metrics)]
#[metrics(scope = 123)]
struct CustomMetrics5;

#[derive(Metrics)]
#[metrics(scope = "some-scope")]
struct CustomMetrics6;

#[derive(Metrics)]
#[metrics(scope = "some_scope", scope = "another_scope")]
struct CustomMetrics7;

#[derive(Metrics)]
#[metrics(separator = value)]
struct CustomMetrics8;

#[derive(Metrics)]
#[metrics(separator = 123)]
struct CustomMetrics9;

#[derive(Metrics)]
#[metrics(separator = "x")]
struct CustomMetrics10;

#[derive(Metrics)]
#[metrics(separator = "_", separator = ":")]
struct CustomMetrics11;

#[derive(Metrics)]
#[metrics(random = "value")]
struct CustomMetrics12;

#[derive(Metrics)]
#[metrics(scope = "scope", dynamic = true)]
struct CustomMetrics13;

#[derive(Metrics)]
#[metrics(scope = "scope", labels = "not_an_array")]
struct CustomMetrics14;

#[derive(Metrics)]
#[metrics(scope = "scope", labels = [], labels = [])]
struct CustomMetrics15;

#[derive(Metrics)]
#[metrics(scope = "scope", labels = [("key", "value1"), ("key", "value2")])]
struct CustomMetrics16;
