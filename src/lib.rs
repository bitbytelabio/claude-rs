pub mod error;
pub mod utils;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

use reqwest::header::{ HeaderValue, HeaderMap, ACCEPT, ORIGIN, REFERER, COOKIE };
use tracing::{ info, debug, warn, error };
use serde::Deserialize;
#[derive(Debug)]
pub struct Client {
    pub org_uuid: String,
    pub cookies: String,
}

#[derive(Debug, Deserialize)]
pub struct Conversation {
    pub uuid: String,
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub uuid: String,
    pub attachments: Vec<Attachment>,
    pub sender: String,
    pub index: i32,
    pub text: String,
    #[serde(default)]
    pub chat_feedback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub extracted_content: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
}

static UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36";

fn build_request(cookie: Option<&str>) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://claude.ai"));
    headers.insert(REFERER, HeaderValue::from_static("https://claude.ai/chats/"));
    if let Some(cookie) = cookie {
        headers.insert(COOKIE, HeaderValue::from_str(cookie)?);
    }

    let client = reqwest::Client
        ::builder()
        .use_rustls_tls()
        .default_headers(headers)
        .https_only(true)
        .user_agent(UA)
        .gzip(true)
        .build()?;
    Ok(client)
}

impl Client {
    pub async fn new(cookies: String) -> Self {
        let org_uuid = match Self::get_organization_id(cookies.clone()).await {
            Ok(id) => id,
            Err(e) => {
                error!("failed to get organization id: {}, cookies are expired or invalid", e);
                std::process::exit(1);
            }
        };
        Self { cookies, org_uuid }
    }
    pub async fn get_organization_id(cookies: String) -> Result<String> {
        let url = "https://claude.ai/api/organizations";

        #[derive(Deserialize, Debug)]
        struct Response {
            uuid: String,
        }

        let res: Vec<Response> = build_request(Some(&cookies))?
            .get(url)
            .send().await
            .unwrap()
            .json().await?;

        debug!("response: {:#?}", res);

        Ok(res[0].uuid.clone())
    }

    pub async fn create_new_chat(&self) {
        let url = format!(
            "https://claude.ai/api/organizations/{}/chat_conversations",
            self.org_uuid
        );
        todo!()
    }

    pub async fn list_all_conversations(&self) -> Result<Vec<Conversation>> {
        let url = format!(
            "https://claude.ai/api/organizations/{}/chat_conversations",
            self.org_uuid
        );
        let res: Vec<Conversation> = build_request(Some(&self.cookies))?
            .get(url)
            .send().await
            .unwrap()
            .json().await
            .unwrap();

        debug!("response: {:#?}", res);

        Ok(res)
    }

    pub async fn chat_conversation_history(&self, chat_uuid: &str) -> Result<Vec<ChatMessage>> {
        let url = format!(
            "https://claude.ai/api/organizations/{}/chat_conversations/{}",
            self.org_uuid,
            chat_uuid
        );

        #[derive(Deserialize, Debug)]
        struct Response {
            chat_messages: Vec<ChatMessage>,
        }

        let res: Response = build_request(Some(&self.cookies))?
            .get(url)
            .send().await
            .unwrap()
            .json().await?;

        debug!("response: {:#?}", res.chat_messages);

        Ok(res.chat_messages)
    }
}
