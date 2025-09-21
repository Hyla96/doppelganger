use clap::Parser;
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;
use warp::{reply, Filter, Reply};

const DEFAULT_PORT: u16 = 3001;
const DELAY_SECONDS: u64 = 1;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

async fn example_handler() -> Result<impl Reply, Infallible> {
    println!("Called example endpoint");

    sleep(Duration::from_secs(DELAY_SECONDS)).await;

    let response = json!({
        "message": "V1 service response",
        "delay": format!("{} second", DELAY_SECONDS),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(reply::json(&response))
}

async fn health_handler() -> Result<impl Reply, Infallible> {
    let response = json!({
        "status": "healthy"
    });
    Ok(reply::json(&response))
}

async fn root_handler() -> Result<impl Reply, Infallible> {
    Ok(reply::with_status(
        "Service is running. Try /example-endpoint",
        warp::http::StatusCode::OK,
    ))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let health_route = warp::path("health")
        .and(warp::get())
        .and_then(health_handler);

    let example_route = warp::path("example-endpoint")
        .and(warp::get())
        .and_then(example_handler);

    let root_route = warp::path::end().and(warp::get()).and_then(root_handler);

    let routes = root_route.or(health_route).or(example_route);

    println!("V1 service listening on http://0.0.0.0:{}", args.port);

    warp::serve(routes).run(([0, 0, 0, 0], args.port)).await;
}
