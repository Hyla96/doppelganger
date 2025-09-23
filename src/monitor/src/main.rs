use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, error};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::Message;
use tokio::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone)]
pub struct AppState {
    pub kafka_consumer: Arc<StreamConsumer>,
    pub shutdown: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestLog {
    pub request_id: String,
    pub service_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub path: String,
    pub request_headers: std::collections::HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_headers: std::collections::HashMap<String, String>,
    pub response_body: String,
    pub response_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvoyAccessLog {
    // Envoy access log format
    pub start_time: String,
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub response_code: u16,
    pub response_flags: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub duration: u64,
    pub upstream_service_time: Option<u64>,
    pub x_forwarded_for: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub authority: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("monitor=debug,tower_http=debug")
        .init();

    info!("Starting Monitor service for primary service observability");

    // Initialize Kafka consumer
    let kafka_brokers = std::env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "kafka:9092".to_string());

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "monitor-service")
        .set("bootstrap.servers", &kafka_brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["relay-logs"])
        .expect("Can't subscribe to specified topics");

    let shutdown = Arc::new(AtomicBool::new(false));

    let state = AppState {
        kafka_consumer: Arc::new(consumer),
        shutdown: shutdown.clone(),
    };

    // Start Kafka consumer task
    let consumer_state = state.clone();
    tokio::spawn(async move {
        kafka_consumer_task(consumer_state).await;
    });

    // Build application
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/log/access", post(receive_access_log))
        .route("/log/request", post(receive_request_log))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9090").await?;
    info!("Monitor service listening on http://0.0.0.0:9090");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Monitor is healthy"
}

async fn receive_access_log(
    State(_state): State<Arc<AppState>>,
    Json(log): Json<EnvoyAccessLog>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    info!("Received access log: {} {} -> {}", log.method, log.path, log.response_code);

    // TODO: Store access log to database for primary service monitoring
    // This provides zero-latency monitoring of primary service traffic

    Ok(Json(serde_json::json!({"status": "logged"})))
}

async fn receive_request_log(
    State(_state): State<Arc<AppState>>,
    Json(log): Json<RequestLog>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    info!("Received request log: {} {} -> {} ({}ms)",
          log.method, log.path, log.response_status, log.response_time_ms);

    // TODO: Store detailed request log to database
    // This can be used for comparison with shadow service logs

    Ok(Json(serde_json::json!({"status": "logged"})))
}

async fn kafka_consumer_task(state: AppState) {
    info!("Starting Kafka consumer task for relay logs");

    loop {
        if state.shutdown.load(Ordering::Relaxed) {
            info!("Shutting down Kafka consumer");
            break;
        }

        match state.kafka_consumer.recv().await {
            Ok(m) => {
                if let Some(payload) = m.payload_view::<str>() {
                    match payload {
                        Ok(text) => {
                            match serde_json::from_str::<RelayLog>(text) {
                                Ok(relay_log) => {
                                    info!("Received relay log from Kafka: request_id={}, method={}, path={}, status={}, response_time={}ms",
                                          relay_log.request_id, relay_log.method, relay_log.path,
                                          relay_log.response_status, relay_log.response_time_ms);

                                    // TODO: Process and store relay log for comparison
                                    if let Err(e) = process_relay_log(&relay_log).await {
                                        error!("Failed to process relay log: {}", e);
                                    }
                                }
                                Err(e) => error!("Failed to deserialize relay log: {}", e),
                            }
                        }
                        Err(e) => error!("Failed to parse Kafka message payload: {:?}", e),
                    }
                } else {
                    error!("Received empty Kafka message");
                }
            }
            Err(e) => {
                error!("Kafka receive error: {:?}", e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelayLog {
    pub request_id: String,
    pub service_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub path: String,
    pub request_headers: std::collections::HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_headers: std::collections::HashMap<String, String>,
    pub response_body: String,
    pub response_time_ms: u64,
}

async fn process_relay_log(relay_log: &RelayLog) -> anyhow::Result<()> {
    // TODO: Compare with primary service responses
    // TODO: Store comparison results
    // TODO: Generate alerts for significant differences

    info!("Processing relay log for comparison: {}", relay_log.request_id);
    Ok(())
}

// TODO: Implement storage and comparison functions:

// async fn store_access_log(log: &EnvoyAccessLog) -> anyhow::Result<()> {
//     // Store lightweight access logs from Envoy for primary service
// }

// async fn store_request_log(log: &RequestLog) -> anyhow::Result<()> {
//     // Store detailed request logs for comparison
// }

// async fn compare_responses(primary: &RequestLog, shadow: &RelayLog) -> anyhow::Result<()> {
//     // Compare primary and shadow service responses
// }

// async fn analyze_primary_performance() -> anyhow::Result<()> {
//     // Analyze primary service performance without affecting latency
// }