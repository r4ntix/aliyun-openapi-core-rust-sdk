use aliyun_openapi_core_rust_sdk::client::roa::ROAClient;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "http://mt.aliyuncs.com",
    );

    // create params.
    let mut params = HashMap::new();
    params.insert("SourceText", "你好");
    params.insert("SourceLanguage", "zh");
    params.insert("TargetLanguage", "en");
    params.insert("FormatType", "text");
    params.insert("Scene", "general");

    // call `Translate` with json params.
    let response = aliyun_openapi_client
        .version("2018-04-08")
        .post("/api/translate/web/general")
        .header([("Content-Type".to_string(), "application/json".to_string())])?
        .body(json!(params).to_string())?
        .text()
        .await?;

    println!("Translate response:\n{response}");

    Ok(())
}
