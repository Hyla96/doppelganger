use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, error};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tokio::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub target_service_url: String,
    pub client: reqwest::Client,
    pub kafka_producer: Arc<FutureProducer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelayLog {
    pub request_id: String,
    pub service_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub path: String,
    pub request_headers: HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_headers: HashMap<String, String>,
    pub response_body: String,
    pub response_time_ms: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("relay=debug,tower_http=debug")
        .init();

    info!("Starting Relay service");

    // Configuration
    let target_service_url = std::env::var("TARGET_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());
    let service_name = std::env::var("SERVICE_NAME")
        .unwrap_or_else(|_| "unknown".to_string());

    // Initialize Kafka producer
    let kafka_brokers = std::env::var("KAFKA_BROKERS")
        .unwrap_or_else(|_| "kafka:9092".to_string());

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let state = AppState {
        target_service_url: target_service_url.clone(),
        client: reqwest::Client::new(),
        kafka_producer: Arc::new(producer),
    };

    info!("Target service: {}", target_service_url);
    info!("Service name: {}", service_name);

    // Build application
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/*path", get(relay_get).post(relay_post))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("Relay listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Relay is healthy"
}

async fn relay_get(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    relay_request(state, "GET".to_string(), path, params, headers, None).await
}

async fn relay_post(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<serde_json::Value>, StatusCode> {
    relay_request(state, "POST".to_string(), path, params, headers, Some(body)).await
}

async fn relay_request(
    state: Arc<AppState>,
    method: String,
    path: String,
    params: HashMap<String, String>,
    headers: HeaderMap,
    body: Option<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let start_time = std::time::Instant::now();

    info!("Relaying {} request to /{} (request_id: {})", method, path, request_id);

    // Build URL with query parameters
    let mut url = format!("{}/{}", state.target_service_url, path);
    if !params.is_empty() {
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        url = format!("{}?{}", url, query_string);
    }

    // Convert headers
    let mut request_headers = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            request_headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // Make request to target service
    let mut request_builder = match method.as_str() {
        "GET" => state.client.get(&url),
        "POST" => state.client.post(&url),
        "PUT" => state.client.put(&url),
        "DELETE" => state.client.delete(&url),
        "PATCH" => state.client.patch(&url),
        _ => {
            error!("Unsupported HTTP method: {}", method);
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }
    };

    // Add headers (filter out problematic ones)
    for (name, value) in headers.iter() {
        let name_str = name.as_str().to_lowercase();
        // Skip headers that can cause issues
        if !["host", "content-length", "connection", "upgrade", "proxy-connection"].contains(&name_str.as_str()) {
            request_builder = request_builder.header(name, value);
        }
    }

    // Add body if present
    if let Some(body_content) = &body {
        request_builder = request_builder.body(body_content.clone());
    }

    // Execute request
    match request_builder.send().await {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            let status = response.status();

            // Collect response headers
            let mut response_headers = HashMap::new();
            for (name, value) in response.headers().iter() {
                if let Ok(value_str) = value.to_str() {
                    response_headers.insert(name.to_string(), value_str.to_string());
                }
            }

            // Get response body
            let response_body = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    error!("Failed to read response body: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Log the relay information (this is where you'll implement actual logging/storage)
            let relay_log = RelayLog {
                request_id: request_id.clone(),
                service_name: std::env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string()),
                timestamp: chrono::Utc::now(),
                method: method.clone(),
                path: path.clone(),
                request_headers,
                request_body: body,
                response_status: status.as_u16(),
                response_headers,
                response_body: response_body.clone(),
                response_time_ms: elapsed.as_millis() as u64,
            };

            // Send relay log to Kafka
            if let Err(e) = send_to_kafka(&state.kafka_producer, &relay_log).await {
                error!("Failed to send relay log to Kafka: {}", e);
            }

            info!("Relay log: request_id={}, status={}, response_time={}ms",
                  request_id, status.as_u16(), elapsed.as_millis());

            // Try to parse response as JSON, fallback to string
            let json_response = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_body) {
                json
            } else {
                serde_json::json!({ "data": response_body })
            };

            Ok(Json(json_response))
        }
        Err(e) => {
            error!("Failed to relay request to {}: {}", url, e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

async fn send_to_kafka(producer: &FutureProducer, log: &RelayLog) -> anyhow::Result<()> {
    let topic = "relay-logs";
    let key = log.request_id.clone();
    let payload = serde_json::to_string(log)?;

    let record = FutureRecord::to(topic)
        .key(&key)
        .payload(&payload);

    match producer.send(record, Duration::from_secs(0)).await {
        Ok(_) => {
            info!("Successfully sent relay log to Kafka: {}", key);
            Ok(())
        }
        Err((kafka_error, _)) => {
            error!("Failed to send to Kafka: {:?}", kafka_error);
            Err(anyhow::anyhow!("Kafka send error: {:?}", kafka_error))
        }
    }
}

// TODO: Implement additional storage functions:

// async fn store_relay_log(log: &RelayLog) -> anyhow::Result<()> {
//     // Store to database for persistence
// }