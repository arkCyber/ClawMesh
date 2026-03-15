//! Prometheus Metrics
//!
//! Provides comprehensive monitoring metrics for aerospace-grade observability.
//! Tracks performance, errors, and system health in real-time.

use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter,
    /// Gauge - value that can go up or down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
}

/// Metric labels
pub type Labels = Vec<(String, String)>;

/// Single metric
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: Labels,
    pub help: String,
}

/// Metrics registry
pub struct MetricsRegistry {
    metrics: Arc<RwLock<Vec<Metric>>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a counter
    pub fn counter(&self, name: &str, help: &str, labels: Labels) -> CounterMetric {
        CounterMetric {
            name: name.to_string(),
            help: help.to_string(),
            labels,
            registry: Arc::clone(&self.metrics),
        }
    }

    /// Register a gauge
    pub fn gauge(&self, name: &str, help: &str, labels: Labels) -> GaugeMetric {
        GaugeMetric {
            name: name.to_string(),
            help: help.to_string(),
            labels,
            registry: Arc::clone(&self.metrics),
        }
    }

    /// Register a histogram
    pub fn histogram(&self, name: &str, help: &str, labels: Labels) -> HistogramMetric {
        HistogramMetric {
            name: name.to_string(),
            help: help.to_string(),
            labels,
            values: Arc::new(RwLock::new(Vec::new())),
            registry: Arc::clone(&self.metrics),
        }
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> String {
        let metrics = self.metrics.read();
        let mut output = String::new();

        for metric in metrics.iter() {
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            output.push_str(&format!("# TYPE {} {:?}\n", metric.name, metric.metric_type));

            let labels_str = if metric.labels.is_empty() {
                String::new()
            } else {
                format!(
                    "{{{}}}",
                    metric.labels
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            };

            match &metric.value {
                MetricValue::Counter(v) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, v));
                }
                MetricValue::Gauge(v) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, v));
                }
                MetricValue::Histogram(values) => {
                    if !values.is_empty() {
                        let sum: f64 = values.iter().sum();
                        let count = values.len();
                        output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum));
                        output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, count));
                    }
                }
            }
        }

        output
    }

    /// Get metrics as JSON
    pub fn export_json(&self) -> String {
        let metrics = self.metrics.read();
        serde_json::to_string_pretty(&*metrics).unwrap_or_default()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Counter metric
pub struct CounterMetric {
    name: String,
    help: String,
    labels: Labels,
    registry: Arc<RwLock<Vec<Metric>>>,
}

impl CounterMetric {
    /// Increment counter by 1
    pub fn inc(&self) {
        self.add(1);
    }

    /// Add value to counter
    pub fn add(&self, value: u64) {
        let mut metrics = self.registry.write();
        
        if let Some(metric) = metrics.iter_mut().find(|m| m.name == self.name && m.labels == self.labels) {
            if let MetricValue::Counter(ref mut v) = metric.value {
                *v += value;
            }
        } else {
            metrics.push(Metric {
                name: self.name.clone(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(value),
                labels: self.labels.clone(),
                help: self.help.clone(),
            });
        }
    }
}

/// Gauge metric
pub struct GaugeMetric {
    name: String,
    help: String,
    labels: Labels,
    registry: Arc<RwLock<Vec<Metric>>>,
}

impl GaugeMetric {
    /// Set gauge value
    pub fn set(&self, value: f64) {
        let mut metrics = self.registry.write();
        
        if let Some(metric) = metrics.iter_mut().find(|m| m.name == self.name && m.labels == self.labels) {
            metric.value = MetricValue::Gauge(value);
        } else {
            metrics.push(Metric {
                name: self.name.clone(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(value),
                labels: self.labels.clone(),
                help: self.help.clone(),
            });
        }
    }

    /// Increment gauge
    pub fn inc(&self) {
        self.add(1.0);
    }

    /// Decrement gauge
    pub fn dec(&self) {
        self.add(-1.0);
    }

    /// Add to gauge
    pub fn add(&self, value: f64) {
        let mut metrics = self.registry.write();
        
        if let Some(metric) = metrics.iter_mut().find(|m| m.name == self.name && m.labels == self.labels) {
            if let MetricValue::Gauge(ref mut v) = metric.value {
                *v += value;
            }
        } else {
            metrics.push(Metric {
                name: self.name.clone(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(value),
                labels: self.labels.clone(),
                help: self.help.clone(),
            });
        }
    }
}

/// Histogram metric
pub struct HistogramMetric {
    name: String,
    help: String,
    labels: Labels,
    values: Arc<RwLock<Vec<f64>>>,
    registry: Arc<RwLock<Vec<Metric>>>,
}

impl HistogramMetric {
    /// Observe a value
    pub fn observe(&self, value: f64) {
        let mut values = self.values.write();
        values.push(value);

        let mut metrics = self.registry.write();
        if let Some(metric) = metrics.iter_mut().find(|m| m.name == self.name && m.labels == self.labels) {
            metric.value = MetricValue::Histogram(values.clone());
        } else {
            metrics.push(Metric {
                name: self.name.clone(),
                metric_type: MetricType::Histogram,
                value: MetricValue::Histogram(values.clone()),
                labels: self.labels.clone(),
                help: self.help.clone(),
            });
        }
    }

    /// Time a function execution
    pub fn time<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed().as_secs_f64();
        self.observe(elapsed);
        result
    }
}

/// Global metrics for ClawMesh
pub struct ClawMeshMetrics {
    pub registry: MetricsRegistry,
    
    // Message metrics
    pub messages_sent_total: CounterMetric,
    pub messages_delivered_total: CounterMetric,
    pub messages_cached_total: CounterMetric,
    pub messages_failed_total: CounterMetric,
    pub message_delivery_duration: HistogramMetric,
    
    // User metrics
    pub users_online: GaugeMetric,
    pub users_total: GaugeMetric,
    
    // Cache metrics
    pub cache_size: GaugeMetric,
    pub cache_hits_total: CounterMetric,
    pub cache_misses_total: CounterMetric,
    
    // Rate limit metrics
    pub rate_limit_exceeded_total: CounterMetric,
    
    // API metrics
    pub http_requests_total: CounterMetric,
    pub http_request_duration: HistogramMetric,
    pub http_errors_total: CounterMetric,
}

impl ClawMeshMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        let registry = MetricsRegistry::new();

        Self {
            messages_sent_total: registry.counter(
                "clawmesh_messages_sent_total",
                "Total number of messages sent",
                vec![],
            ),
            messages_delivered_total: registry.counter(
                "clawmesh_messages_delivered_total",
                "Total number of messages delivered",
                vec![],
            ),
            messages_cached_total: registry.counter(
                "clawmesh_messages_cached_total",
                "Total number of messages cached for offline delivery",
                vec![],
            ),
            messages_failed_total: registry.counter(
                "clawmesh_messages_failed_total",
                "Total number of failed message deliveries",
                vec![],
            ),
            message_delivery_duration: registry.histogram(
                "clawmesh_message_delivery_duration_seconds",
                "Message delivery duration in seconds",
                vec![],
            ),
            users_online: registry.gauge(
                "clawmesh_users_online",
                "Number of currently online users",
                vec![],
            ),
            users_total: registry.gauge(
                "clawmesh_users_total",
                "Total number of registered users",
                vec![],
            ),
            cache_size: registry.gauge(
                "clawmesh_cache_size_bytes",
                "Size of message cache in bytes",
                vec![],
            ),
            cache_hits_total: registry.counter(
                "clawmesh_cache_hits_total",
                "Total number of cache hits",
                vec![],
            ),
            cache_misses_total: registry.counter(
                "clawmesh_cache_misses_total",
                "Total number of cache misses",
                vec![],
            ),
            rate_limit_exceeded_total: registry.counter(
                "clawmesh_rate_limit_exceeded_total",
                "Total number of rate limit violations",
                vec![],
            ),
            http_requests_total: registry.counter(
                "clawmesh_http_requests_total",
                "Total number of HTTP requests",
                vec![("method".to_string(), "GET".to_string())],
            ),
            http_request_duration: registry.histogram(
                "clawmesh_http_request_duration_seconds",
                "HTTP request duration in seconds",
                vec![],
            ),
            http_errors_total: registry.counter(
                "clawmesh_http_errors_total",
                "Total number of HTTP errors",
                vec![],
            ),
            registry,
        }
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> String {
        self.registry.export()
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> String {
        self.registry.export_json()
    }
}

impl Default for ClawMeshMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_metric() {
        let registry = MetricsRegistry::new();
        let counter = registry.counter("test_counter", "Test counter", vec![]);
        
        counter.inc();
        counter.add(5);
        
        let export = registry.export();
        assert!(export.contains("test_counter"));
        assert!(export.contains("6")); // 1 + 5
    }

    #[test]
    fn test_gauge_metric() {
        let registry = MetricsRegistry::new();
        let gauge = registry.gauge("test_gauge", "Test gauge", vec![]);
        
        gauge.set(10.0);
        gauge.inc();
        gauge.dec();
        
        let export = registry.export();
        assert!(export.contains("test_gauge"));
        assert!(export.contains("10")); // 10 + 1 - 1
    }

    #[test]
    fn test_histogram_metric() {
        let registry = MetricsRegistry::new();
        let histogram = registry.histogram("test_histogram", "Test histogram", vec![]);
        
        histogram.observe(1.0);
        histogram.observe(2.0);
        histogram.observe(3.0);
        
        let export = registry.export();
        assert!(export.contains("test_histogram_sum"));
        assert!(export.contains("test_histogram_count"));
        assert!(export.contains("6")); // sum = 1 + 2 + 3
        assert!(export.contains("3")); // count = 3
    }

    #[test]
    fn test_histogram_timer() {
        let registry = MetricsRegistry::new();
        let histogram = registry.histogram("test_timer", "Test timer", vec![]);
        
        let result = histogram.time(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        
        let export = registry.export();
        assert!(export.contains("test_timer"));
    }

    #[test]
    fn test_labels() {
        let registry = MetricsRegistry::new();
        let counter = registry.counter(
            "test_labeled",
            "Test with labels",
            vec![("method".to_string(), "POST".to_string())],
        );
        
        counter.inc();
        
        let export = registry.export();
        assert!(export.contains("method=\"POST\""));
    }
}
