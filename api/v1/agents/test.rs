use serde_json::Value;
use std::env;
use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    // Get database connection
    let supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
    let supabase_key = env::var("SUPABASE_ANON_KEY")
        .or_else(|_| env::var("SUPABASE_SERVICE_ROLE_KEY"))
        .unwrap_or_default();

    let mut debug_info = serde_json::json!({
        "supabase_url_present": !supabase_url.is_empty(),
        "supabase_key_present": !supabase_key.is_empty(),
        "url_prefix": supabase_url.chars().take(30).collect::<String>(),
    });

    if supabase_url.is_empty() || supabase_key.is_empty() {
        debug_info["error"] = Value::String("Missing environment variables".to_string());
        return Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .body(serde_json::to_string(&debug_info)?.into())?);
    }

    // Try a simple query
    let client = postgrest::Postgrest::new(format!("{supabase_url}/rest/v1"))
        .insert_header("apikey", &supabase_key)
        .insert_header("Authorization", format!("Bearer {}", &supabase_key));

    let response = client
        .from("agents")
        .select("name")
        .limit(1)
        .execute()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            debug_info["query_status"] = Value::Number(status.as_u16().into());
            debug_info["response_body"] = Value::String(body);
            debug_info["success"] = Value::Bool(status.is_success());
        }
        Err(e) => {
            debug_info["query_error"] = Value::String(format!("{:?}", e));
        }
    }

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(serde_json::to_string(&debug_info)?.into())?)
}