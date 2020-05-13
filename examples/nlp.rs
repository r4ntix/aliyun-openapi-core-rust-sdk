use aliyun_openapi_core_rust_sdk::ROAClient;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("http://nlp.cn-shanghai.aliyuncs.com"),
        String::from("2018-04-08"),
    );

    // create params.
    let mut params = HashMap::new();
    params.insert("q", "你好");
    params.insert("source", "zh");
    params.insert("target", "en");
    params.insert("format", "text");

    // call `Translate` with json params.
    let response = aliyun_openapi_client
        .post("/nlp/api/translate/standard")
        .header(&[("Content-Type", "application/json")])
        .body(&json!(params).to_string())
        .send()?;
    println!("Translate response: {}", response);

    Ok(())
}
