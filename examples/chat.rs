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
    info!("client: {:?}", client);

    // client.list_all_conversations().await.unwrap();
    client.chat_conversation_history("fa1b2c80-d5d9-4855-a4dd-10b48d82f5ee").await.unwrap();
}
