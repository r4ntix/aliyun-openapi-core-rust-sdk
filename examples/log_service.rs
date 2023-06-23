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

    // call `ListProject` api.
    let response = aliyun_openapi_client.clone().get("/").text().await?;
    println!("ListProject response:\n{response}\n");

    // call `GetProject` api.
    let response = aliyun_openapi_client
        .get("/")
        .project("project_name")
        .text()
        .await?;
    println!("GetProject response:\n{response}\n");

    Ok(())
}
