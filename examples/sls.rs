use aliyun_openapi_core_rust_sdk::client::log_service::LogServiceClient;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create log service api client.
    let aliyun_openapi_client = LogServiceClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://cn-hangzhou.log.aliyuncs.com",
    );

    // call `GetLogStore` api.
    let response = aliyun_openapi_client
        .get(format!("/logstores/{}", "logstore"))
        .project("project")
        .text()
        .await?;
    println!("GetLogStore response:\n{response}\n");

    Ok(())
}
