use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let response_body = json!({
        "status": "healthy",
        "service": "carp-api",
        "environment": "serverless",
        "message": "API is running on Vercel",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(response_body.to_string().into())?;

    Ok(response)
}
