pub mod error;

use reqwest::blocking::multipart;
use reqwest::header::{ HeaderValue, HeaderMap, ACCEPT, ORIGIN, REFERER, COOKIE };
use tracing::{ debug, error };
use serde::Deserialize;
use std::time::Duration;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

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
    pub index: usize,
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

    pub async fn create_new_chat(&self) -> Result<Conversation> {
        let url = format!(
            "https://claude.ai/api/organizations/{}/chat_conversations",
            self.org_uuid
        );

        let payload =
            serde_json::json!({
            "uuid": uuid::Uuid::new_v4(),
            "name": "".to_string(),
        });

        let res: Conversation = build_request(Some(&self.cookies))?
            .post(url)
            .json(&payload)
            .send().await?
            .json().await?;

        debug!("response: {:#?}", res);

        Ok(res)
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

    pub async fn delete_conversation(&self, chat_uuid: &str) -> Result<()> {
        let url = format!(
            "https://claude.ai/api/organizations/{}/chat_conversations/{}",
            self.org_uuid,
            chat_uuid
        );

        let payload =
            serde_json::json!({
            "conversation_id": chat_uuid.to_string(),
            });

        let res = build_request(Some(&self.cookies))?.delete(url).json(&payload).send().await?;

        debug!("response: {:#?}", res);

        Ok(())
    }

    pub async fn reset_all(&self) -> Result<()> {
        let conversations = self.list_all_conversations().await?;

        for conversation in conversations {
            self.delete_conversation(&conversation.uuid).await?;
        }
        Ok(())
    }

    pub async fn upload_attachment(&self, file_path: &str) -> Result<()> {
        let url = "https://claude.ai/api/convert_document";

        let form = multipart::Form
            ::new()
            .file("file", file_path)?
            .text("orgUuid", self.org_uuid.clone());

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(ORIGIN, HeaderValue::from_static("https://claude.ai"));
        headers.insert(REFERER, HeaderValue::from_static("https://claude.ai/chats/"));
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookies)?);

        let res = reqwest::blocking::Client
            ::new()
            .post(url)
            .headers(headers)
            .multipart(form)
            .send()?;
        // .json::<serde_json::Value>()?;

        debug!("response: {:#?}", res);

        Ok(())
    }

    pub async fn send_message(
        self,
        chat_uuid: &str,
        prompt: &str,
        attachments: Option<Vec<&str>>,
        timeout: Option<u64>
    ) -> Result<String> {
        let url = "https://claude.ai/api/append_message";
        let attachments = attachments.unwrap_or_default();
        let timeout = timeout.unwrap_or(500);

        let payload =
            serde_json::json!({
             "completion": {
                "prompt": prompt,
                "timezone": "Asia/Kolkata",
                "model": "claude-2"
            },
            "organization_uuid": self.org_uuid.clone(),
            "conversation_uuid": chat_uuid,
            "text": prompt,
            "attachments": attachments
            });

        let response = build_request(Some(&self.cookies))?
            .post(url)
            .json(&payload)
            .timeout(Duration::from_secs(timeout))
            .send().await?;

        let decoded_data = response.text().await?;
        let re = regex::Regex::new(r"\n+").unwrap();
        let decoded_data = re.replace_all(&decoded_data, "\n").trim().to_string();

        let data_strings: Vec<&str> = decoded_data.split('\n').collect();
        let mut completions = Vec::new();

        for data_string in data_strings {
            let json_str = &data_string[6..].trim();
            let data: serde_json::Value = serde_json::from_str(json_str)?;
            debug!("data: {:#?}", data);
            if data.get("completion").is_some() {
                completions.push(data["completion"].as_str().unwrap().to_string());
            }
        }

        let answer = completions.join("");

        debug!("response: {:#?}", answer);

        Ok(answer)
    }

    pub async fn rename_chat(&self, chat_uuid: &str, title: &str) -> Result<()> {
        let url = "https://claude.ai/api/rename_chat";

        let payload =
            serde_json::json!( {
            "organization_uuid": self.org_uuid.clone(),
            "conversation_uuid": chat_uuid.to_string(),
            "title": title.to_string(),
        });

        let res = build_request(Some(&self.cookies))?.post(url).json(&payload).send().await?;

        debug!("response: {:#?}", res);

        Ok(())
    }
}
