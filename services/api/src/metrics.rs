use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::http::StatusCode;

/// Process-local operational counters. Labels are deliberately selected from a
/// fixed list so requests can never create a metric containing a guardian,
/// class, offer token, or other user-provided value.
#[derive(Clone, Default)]
pub struct AppMetrics {
    inner: Arc<Mutex<HttpMetrics>>,
}

#[derive(Default)]
struct HttpMetrics {
    routes: BTreeMap<&'static str, RouteMetrics>,
}

#[derive(Default)]
struct RouteMetrics {
    requests: u64,
    server_errors: u64,
    total_latency_ms: u64,
    max_latency_ms: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct WorkspaceMetrics {
    pub calendar_job_lag_seconds: i64,
    pub unresolved_discrepancies: i64,
    pub offers_created: i64,
    pub offers_accepted: i64,
}

impl AppMetrics {
    pub fn record(&self, route: &'static str, status: StatusCode, elapsed: Duration) {
        let Ok(mut metrics) = self.inner.lock() else {
            // Metrics must never make a booking or a health check unavailable.
            return;
        };
        let entry = metrics.routes.entry(route).or_default();
        entry.requests = entry.requests.saturating_add(1);
        if status.is_server_error() {
            entry.server_errors = entry.server_errors.saturating_add(1);
        }
        let elapsed_ms = elapsed.as_millis().min(u128::from(u64::MAX)) as u64;
        entry.total_latency_ms = entry.total_latency_ms.saturating_add(elapsed_ms);
        entry.max_latency_ms = entry.max_latency_ms.max(elapsed_ms);
    }

    pub fn prometheus(&self, workspace: WorkspaceMetrics) -> String {
        let routes = self
            .inner
            .lock()
            .map(|metrics| {
                metrics
                    .routes
                    .iter()
                    .map(|(route, values)| {
                        (
                            *route,
                            values.requests,
                            values.server_errors,
                            values.total_latency_ms,
                            values.max_latency_ms,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let conversion = if workspace.offers_created == 0 {
            0.0
        } else {
            workspace.offers_accepted as f64 / workspace.offers_created as f64
        };

        let mut output = String::from(
            "# HELP cct_http_requests_total Completed HTTP requests by fixed route group.\n\
             # TYPE cct_http_requests_total counter\n\
             # HELP cct_http_server_errors_total Completed 5xx HTTP responses by fixed route group.\n\
             # TYPE cct_http_server_errors_total counter\n\
             # HELP cct_http_request_duration_milliseconds Request duration totals by fixed route group.\n\
             # TYPE cct_http_request_duration_milliseconds summary\n",
        );
        for (route, requests, server_errors, total_latency_ms, max_latency_ms) in routes {
            output.push_str(&format!(
                "cct_http_requests_total{{route=\"{route}\"}} {requests}\n\
                 cct_http_server_errors_total{{route=\"{route}\"}} {server_errors}\n\
                 cct_http_request_duration_milliseconds_sum{{route=\"{route}\"}} {total_latency_ms}\n\
                 cct_http_request_duration_milliseconds_count{{route=\"{route}\"}} {requests}\n\
                 cct_http_request_duration_milliseconds_max{{route=\"{route}\"}} {max_latency_ms}\n"
            ));
        }
        output.push_str(&format!(
            "# HELP cct_calendar_job_lag_seconds Seconds that the oldest due calendar check is late for this workspace.\n\
             # TYPE cct_calendar_job_lag_seconds gauge\n\
             cct_calendar_job_lag_seconds {}\n\
             # HELP cct_unresolved_capacity_discrepancies Current class counts that need attention in this workspace.\n\
             # TYPE cct_unresolved_capacity_discrepancies gauge\n\
             cct_unresolved_capacity_discrepancies {}\n\
             # HELP cct_released_seat_offers_total Released-seat offers in this workspace by outcome.\n\
             # TYPE cct_released_seat_offers_total gauge\n\
             cct_released_seat_offers_total{{status=\"created\"}} {}\n\
             cct_released_seat_offers_total{{status=\"accepted\"}} {}\n\
             # HELP cct_released_seat_offer_conversion_ratio Accepted released-seat offers divided by created offers.\n\
             # TYPE cct_released_seat_offer_conversion_ratio gauge\n\
             cct_released_seat_offer_conversion_ratio {:.6}\n",
            workspace.calendar_job_lag_seconds.max(0),
            workspace.unresolved_discrepancies.max(0),
            workspace.offers_created.max(0),
            workspace.offers_accepted.max(0),
            conversion,
        ));
        output
    }
}

pub fn route_group(path: &str) -> &'static str {
    match path {
        "/health" => "health",
        "/api/metrics" | "/api/workspaces/metrics" => "metrics",
        path if path.starts_with("/api/demo/") => "demo_api",
        path if path.starts_with("/api/workspaces/") || path == "/api/workspaces" => {
            "workspace_api"
        }
        path if path.starts_with("/api/classes/") => "public_booking_api",
        path if path.starts_with("/api/offers/") => "offer_api",
        path if path.starts_with("/api/") => "api_other",
        path if path.starts_with("/assets/") => "assets",
        _ => "web",
    }
}

#[cfg(test)]
mod tests {
    use super::{route_group, AppMetrics, WorkspaceMetrics};
    use axum::http::StatusCode;
    use std::time::Duration;

    #[test]
    fn uses_fixed_route_groups_and_never_echoes_path_values() {
        assert_eq!(
            route_group("/api/classes/class_secret/book"),
            "public_booking_api"
        );
        assert_eq!(route_group("/offer/offer_secret"), "web");

        let metrics = AppMetrics::default();
        metrics.record(
            "public_booking_api",
            StatusCode::CREATED,
            Duration::from_millis(14),
        );
        let rendered = metrics.prometheus(WorkspaceMetrics {
            calendar_job_lag_seconds: 0,
            unresolved_discrepancies: 1,
            offers_created: 2,
            offers_accepted: 1,
        });
        assert!(rendered.contains("cct_http_requests_total{route=\"public_booking_api\"} 1"));
        assert!(!rendered.contains("class_secret"));
        assert!(!rendered.contains("offer_secret"));
    }
}
