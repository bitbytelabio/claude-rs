use claude::Client;
use std::env::var;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let cookies = format!(
        "activitySessionId={}; sessionKey={}",
        var("SESSION_ID").unwrap(),
        var("SESSION_KEY").unwrap()
    );
    let client = Client::new(cookies).await;
    client
        .send_message(
            "e56a5ab3-0eca-4a04-9c63-3fadaf14cd17",
            "Help me improve this CV",
            Some(vec!["tmp/cv.pdf"]),
            None
        ).await
        .unwrap();
}
