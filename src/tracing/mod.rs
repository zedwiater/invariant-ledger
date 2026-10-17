//! v0.2: Request-scoped tracing hooks for sub-second LLM attribution.
//!
//! Bypasses the 1Hz cgroup poll for per-request energy measurement.
//! Uses request metadata headers to inject energy deltas directly.
//!
//! The core insight: for LLM inference, we don't need to poll hardware
//! at 1Hz and guess attribution. We can measure the energy delta of a
//! single request by sampling telemetry before and after, scoped to
//! the request's lifetime.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A single traced request with its energy measurement.
#[derive(Debug, Clone)]
pub struct TracedRequest {
    pub request_id: String,
    pub user_id: String,
    pub model: String,
    pub started: Instant,
    pub ended: Option<Instant>,
    /// Joules consumed during this request (measured, not estimated)
    pub joules: f64,
    /// Token count (for J/token calculation)
    pub tokens_in: u32,
    pub tokens_out: u32,
    /// Telemetry readings at start/end
    pub telemetry_start: HashMap<String, f64>,
    pub telemetry_end: HashMap<String, f64>,
}

impl TracedRequest {
    pub fn new(request_id: &str, user_id: &str, model: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            user_id: user_id.to_string(),
            model: model.to_string(),
            started: Instant::now(),
            ended: None,
            joules: 0.0,
            tokens_in: 0,
            tokens_out: 0,
            telemetry_start: HashMap::new(),
            telemetry_end: HashMap::new(),
        }
    }

    /// Record telemetry reading at request start.
    /// Call this BEFORE inference begins.
    pub fn begin_trace(&mut self, readings: HashMap<String, f64>) {
        self.telemetry_start = readings;
        self.started = Instant::now();
    }

    /// Record telemetry reading at request end, compute Joules.
    /// Call this AFTER inference completes.
    pub fn end_trace(&mut self, readings: HashMap<String, f64>,
                     tokens_in: u32, tokens_out: u32) {
        self.telemetry_end = readings;
        self.ended = Some(Instant::now());
        self.tokens_in = tokens_in;
        self.tokens_out = tokens_out;

        // Compute energy delta from telemetry
        // For RAPL: readings are cumulative microjoules
        // For GPU: readings are instantaneous watts (integrate over duration)
        self.joules = self.compute_delta();
    }

    fn compute_delta(&self) -> f64 {
        let mut total = 0.0;
        for (source, end_val) in &self.telemetry_end {
            if let Some(start_val) = self.telemetry_start.get(source) {
                if source.contains("rapl") || source.contains("energy") {
                    // Cumulative counter: delta in microjoules
                    let delta_uj = end_val - start_val;
                    // Handle wraparound (assume 64-bit)
                    let delta_uj = if delta_uj < 0.0 { delta_uj + 2f64.powi(63) } else { delta_uj };
                    total += delta_uj / 1e6;
                } else if source.contains("power") || source.contains("watt") {
                    // Instantaneous watts: average × duration
                    let start_w = start_val / 1e6; // assume microwatts
                    let end_w = end_val / 1e6;
                    let avg_w = (start_w + end_w) / 2.0;
                    let duration = self.duration().as_secs_f64();
                    total += avg_w * duration;
                }
            }
        }
        total
    }

    pub fn duration(&self) -> Duration {
        self.ended.unwrap_or_else(Instant::now).duration_since(self.started)
    }

    pub fn joules_per_token(&self) -> f64 {
        let total_tokens = self.tokens_in + self.tokens_out;
        if total_tokens == 0 { 0.0 } else { self.joules / total_tokens as f64 }
    }

    pub fn joules_per_output_token(&self) -> f64 {
        if self.tokens_out == 0 { 0.0 } else { self.joules / self.tokens_out as f64 }
    }

    /// Generate the response headers for this request.
    /// These are the "Joule receipt" — immutable, auditable.
    pub fn response_headers(&self) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("X-Joules-Used".to_string(), format!("{:.4}", self.joules));
        h.insert("X-Joules-Per-Token".to_string(), format!("{:.6}", self.joules_per_token()));
        h.insert("X-Request-Id".to_string(), self.request_id.clone());
        h.insert("X-Model".to_string(), self.model.clone());
        h.insert("X-Duration-Ms".to_string(), format!("{}", self.duration().as_millis()));
        h
    }
}

/// In-memory trace store for active requests.
/// In production, this would be backed by Redis for fleet-wide aggregation.
pub struct TraceStore {
    active: HashMap<String, TracedRequest>,
    completed: Vec<TracedRequest>,
    max_completed: usize,
}

impl TraceStore {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
            completed: Vec::new(),
            max_completed: 10000,
        }
    }

    pub fn begin(&mut self, req: TracedRequest) {
        self.active.insert(req.request_id.clone(), req);
    }

    pub fn end(&mut self, request_id: &str) -> Option<TracedRequest> {
        if let Some(mut req) = self.active.remove(request_id) {
            // Caller should have already called end_trace with readings
            self.completed.push(req.clone());
            if self.completed.len() > self.max_completed {
                self.completed.remove(0);
            }
            Some(req)
        } else {
            None
        }
    }

    /// Per-user Joule aggregation (for rate limiting)
    pub fn user_joules(&self, user_id: &str, window: Duration) -> f64 {
        let cutoff = Instant::now() - window;
        self.completed.iter()
            .filter(|r| r.user_id == user_id)
            .filter(|r| r.started > cutoff)
            .map(|r| r.joules)
            .sum()
    }

    /// Per-model efficiency ranking
    pub fn model_efficiency(&self) -> HashMap<String, f64> {
        let mut by_model: HashMap<String, (f64, u32)> = HashMap::new();
        for r in &self.completed {
            let e = by_model.entry(r.model.clone()).or_insert((0.0, 0));
            e.0 += r.joules;
            e.1 += r.tokens_in + r.tokens_out;
        }
        by_model.into_iter()
            .map(|(m, (j, t))| (m, if t > 0 { j / t as f64 } else { 0.0 }))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_measures_joules() {
        let mut req = TracedRequest::new("req-1", "user-1", "gemma-3-4b");
        let mut start = HashMap::new();
        start.insert("rapl_energy".to_string(), 1000000.0); // 1J in uJ
        req.begin_trace(start);

        std::thread::sleep(Duration::from_millis(10));

        let mut end = HashMap::new();
        end.insert("rapl_energy".to_string(), 2000000.0); // 2J in uJ
        req.end_trace(end, 10, 20);

        assert!((req.joules - 1.0).abs() < 0.01);
        assert!((req.joules_per_token() - 1.0/30.0).abs() < 0.001);
    }

    #[test]
    fn headers_are_receipt() {
        let mut req = TracedRequest::new("req-abc", "user-1", "test");
        let h = req.response_headers();
        assert_eq!(h["X-Request-Id"], "req-abc");
        assert!(h.contains_key("X-Joules-Used"));
    }
}
