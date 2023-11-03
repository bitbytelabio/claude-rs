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

    client.list_all_conversations().await.unwrap();
    // client.chat_conversation_history("a17e16f1-82a4-4b22-9a87-7cc83b2673f2").await.unwrap();
    // client.create_new_chat().await.unwrap();
    // a100c91c-aae5-4bf0-a1bd-48fe9142a617
    // client.delete_conversation("a100c91c-aae5-4bf0-a1bd-48fe9142a617").await.unwrap();
    // client.rename_chat("e56a5ab3-0eca-4a04-9c63-3fadaf14cd17", "test").await.unwrap();
}
