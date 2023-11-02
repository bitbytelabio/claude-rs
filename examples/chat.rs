use claude::Client;
use std::env::var;
use tracing::info;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cookies = format!(
        "activitySessionId={}; sessionKey={}",
        var("SESSION_ID").unwrap(),
        var("SESSION_KEY").unwrap()
    );
    // info!("cookies: {}", cookies);
    let client = Client::new(cookies).await;
    info!("client: {:?}", client)
}
