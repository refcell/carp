use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Extract query parameters for search functionality
    let query = req.uri().query().unwrap_or("");
    let _search_params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    // Placeholder search response
    let response_body = json!({
        "agents": [],
        "total": 0,
        "page": 1,
        "per_page": 20,
        "message": "Search functionality coming soon",
        "query": query
    });

    let response = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(response_body.to_string().into())?;

    Ok(response)
}
