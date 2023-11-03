pub mod error;

use reqwest::{
    header::{ HeaderValue, HeaderMap, ACCEPT, ORIGIN, REFERER, COOKIE, CONNECTION, USER_AGENT },
    multipart::{ Part, Form },
    Body,
};
use serde_json::Value;
use tokio::fs::File;
use tokio_util::codec::{ BytesCodec, FramedRead };
use tracing::{ debug, error };
use serde::Deserialize;
use std::{ time::Duration, path::Path };

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

lazy_static::lazy_static! {
    static ref HEADERS: HeaderMap = {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(ORIGIN, HeaderValue::from_static("https://claude.ai"));
        headers.insert(REFERER, HeaderValue::from_static("https://claude.ai/chats/"));
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        headers
    };
}

fn build_request(cookie: &str) -> Result<reqwest::Client> {
    let mut headers = HEADERS.clone();
    headers.insert(COOKIE, HeaderValue::from_str(cookie)?);

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
    /// Creates a new instance of the struct.
    ///
    /// This function takes a `cookies` string as input, which is used to get the organization ID.
    /// If the organization ID cannot be retrieved (which may happen if the cookies are expired or invalid),
    /// an error message is logged and the process is terminated with exit code 1.
    ///
    /// # Arguments
    ///
    /// * `cookies` - A string representing the cookies to be used for getting the organization ID.
    ///
    /// # Returns
    ///
    /// * `Self` - An instance of the struct, with the `cookies` field set to the input `cookies` string,
    /// and the `org_uuid` field set to the retrieved organization ID.
    ///
    /// # Errors
    ///
    /// This function will exit the process if the organization ID cannot be retrieved.
    ///
    /// # Examples
    ///
    /// ```
    /// use claude::Client;
    /// use std::env::var;
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv::dotenv().ok();
    ///     tracing_subscriber::fmt::init();
    ///     let cookies = format!(
    ///         "activitySessionId={}; sessionKey={}",
    ///         var("SESSION_ID").unwrap(),
    ///         var("SESSION_KEY").unwrap()
    ///     );
    ///     let client = Client::new(cookies).await;
    ///     tracing::info!("Client created, {:?}", client);
    /// }
    /// ```
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

    /// Retrieves the organization ID from the API.
    ///
    /// This function sends a GET request to the API and deserializes the response into a vector of `Response` structs.
    /// The `uuid` field of the first `Response` struct in the vector is then returned.
    ///
    /// # Arguments
    ///
    /// * `cookies` - A string representing the cookies to be used for the request.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The organization ID, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request fails or if the response cannot be deserialized.
    pub async fn get_organization_id(cookies: String) -> Result<String> {
        let url = "https://claude.ai/api/organizations";

        #[derive(Deserialize, Debug)]
        struct Response {
            uuid: String,
        }

        let res: Vec<Response> = build_request(&cookies)?.get(url).send().await?.json().await?;

        debug!("response: {:#?}", res);

        Ok(res[0].uuid.clone())
    }

    /// Creates a new chat conversation.
    ///
    /// This function sends a POST request to the API to create a new chat conversation.
    /// The payload for the request includes a randomly generated UUID and an empty name.
    ///
    /// # Returns
    ///
    /// * `Result<Conversation>` - The created chat conversation, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request fails or if the response cannot be deserialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use claude::Client;
    /// use std::env::var;
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv::dotenv().ok();
    ///     tracing_subscriber::fmt::init();
    ///     let cookies = format!(
    ///         "activitySessionId={}; sessionKey={}",
    ///         var("SESSION_ID").unwrap(),
    ///         var("SESSION_KEY").unwrap()
    ///     );
    ///     let client = Client::new(cookies).await;
    ///     let chat = client.create_new_chat().await.unwrap();
    ///     tracing::info!("{:?}", chat);
    /// }
    /// ```
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

        let res: Conversation = build_request(&self.cookies)?
            .post(url)
            .json(&payload)
            .send().await?
            .json().await?;

        debug!("response: {:#?}", res);

        Ok(res)
    }

    /// Lists all chat conversations.
    ///
    /// This function sends a GET request to the API to retrieve all chat conversations for the organization.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Conversation>>` - A vector of `Conversation` structs, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request fails or if the response cannot be deserialized.
    ///
    /// # Examples
    ///
    /// # Examples
    ///
    /// ```
    /// use claude::Client;
    /// use std::env::var;
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv::dotenv().ok();
    ///     tracing_subscriber::fmt::init();
    ///     let cookies = format!(
    ///         "activitySessionId={}; sessionKey={}",
    ///         var("SESSION_ID").unwrap(),
    ///         var("SESSION_KEY").unwrap()
    ///     );
    ///     let client = Client::new(cookies).await;
    ///     let chats = client.list_all_conversations().await.unwrap();
    ///     tracing::info!("{:?}", chats);
    /// }
    /// ```
    pub async fn list_all_conversations(&self) -> Result<Vec<Conversation>> {
        let url = format!(
            "https://claude.ai/api/organizations/{}/chat_conversations",
            self.org_uuid
        );
        let res: Vec<Conversation> = build_request(&self.cookies)?
            .get(url)
            .send().await?
            .json().await?;

        debug!("response: {:#?}", res);

        Ok(res)
    }

    /// Retrieves the history of a chat conversation.
    ///
    /// This function sends a GET request to the API to retrieve the history of a chat conversation.
    /// The history is returned as a vector of `ChatMessage` structs.
    ///
    /// # Arguments
    ///
    /// * `chat_uuid` - A string representing the UUID of the chat conversation.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<ChatMessage>>` - A vector of `ChatMessage` structs, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request fails or if the response cannot be deserialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use claude::Client;
    /// use std::env::var;
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv::dotenv().ok();
    ///     tracing_subscriber::fmt::init();
    ///     let cookies = format!(
    ///         "activitySessionId={}; sessionKey={}",
    ///         var("SESSION_ID").unwrap(),
    ///         var("SESSION_KEY").unwrap()
    ///     );
    ///     let client = Client::new(cookies).await;
    ///     let chat_hist = client.chat_conversation_history("chat_uuid").await.unwrap();
    ///     tracing::info!("{:#?}", chat_hist);
    /// }
    /// ```
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

        let res: Response = build_request(&self.cookies)?.get(url).send().await?.json().await?;

        debug!("response: {:#?}", res.chat_messages);

        Ok(res.chat_messages)
    }

    /// Deletes a chat conversation.
    ///
    /// This function sends a DELETE request to the API to delete a chat conversation.
    ///
    /// # Arguments
    ///
    /// * `chat_uuid` - A string representing the UUID of the chat conversation to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An empty `Result`, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use claude::Client;
    /// use std::env::var;
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv::dotenv().ok();
    ///     tracing_subscriber::fmt::init();
    ///     let cookies = format!(
    ///         "activitySessionId={}; sessionKey={}",
    ///         var("SESSION_ID").unwrap(),
    ///         var("SESSION_KEY").unwrap()
    ///     );
    ///     let client = Client::new(cookies).await;
    ///     let chat_hist = client.delete_conversation("chat_uuid_string").await.unwrap();
    /// }
    /// ```
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

        let res = build_request(&self.cookies)?.delete(url).json(&payload).send().await?;

        debug!("response: {:#?}", res);

        Ok(())
    }

    /// Resets all chat conversations.
    ///
    /// This function retrieves all chat conversations and deletes each one.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An empty `Result`, if all chat conversations are successfully deleted. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the retrieval of chat conversations fails or if any chat conversation cannot be deleted.
    ///
    pub async fn reset_all(&self) -> Result<()> {
        let conversations = self.list_all_conversations().await?;

        for conversation in conversations {
            self.delete_conversation(&conversation.uuid).await?;
        }
        Ok(())
    }

    /// Uploads an attachment to the API.
    ///
    /// This function sends a POST request to the API to upload a document.
    /// The document is read from the file at the specified path and included in the request as a multipart form data.
    /// The MIME type of the document is determined based on its file extension.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A string representing the path to the file to be uploaded.
    ///
    /// # Returns
    ///
    /// * `Result<Value>` - The API response, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened, if the request fails, or if the response cannot be deserialized.
    ///
    pub async fn upload_attachment(&self, file_path: &str) -> Result<Value> {
        let url = "https://claude.ai/api/convert_document";
        let mut headers = HEADERS.clone();
        headers.insert(COOKIE, HeaderValue::from_str(&self.cookies)?);

        let client = build_request(&self.cookies)?;

        let file = File::open(file_path).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let extension = Path::new(file_path).extension().unwrap().to_str().unwrap();

        let mine = match extension {
            "txt" => "text/plain".to_string(),
            _ => format!("application/{}", extension),
        };
        let part = Part::stream(Body::wrap_stream(stream))
            .file_name(file_path.to_string())
            .mime_str(&mine)?;
        let form = Form::new().part("file", part).text("orgUuid", self.org_uuid.clone());
        let res = client.post(url).multipart(form).send().await?.json::<Value>().await?;
        debug!("response: {:#?}", res);

        Ok(res)
    }

    /// Sends a message to a chat conversation.
    ///
    /// This function sends a POST request to the API to append a message to a chat conversation.
    /// The message can include attachments, which are uploaded to the API before the message is sent.
    /// The function waits for a response from the API for a specified amount of time before timing out.
    ///
    /// # Arguments
    ///
    /// * `chat_uuid` - A string representing the UUID of the chat conversation.
    /// * `prompt` - A string representing the message to be sent.
    /// * `attachments` - An optional vector of strings representing the paths to the files to be uploaded as attachments.
    /// * `timeout` - An optional number representing the amount of time (in seconds) to wait for a response before timing out.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The API response, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if an attachment cannot be uploaded, if the request fails, if the response cannot be deserialized, or if the request times out.
    ///
    pub async fn send_message(
        &self,
        chat_uuid: &str,
        prompt: &str,
        attachments: Option<Vec<&str>>,
        timeout: Option<u64>
    ) -> Result<String> {
        let url = "https://claude.ai/api/append_message";
        let attachments = match attachments {
            Some(attachments) => {
                let mut res: Vec<Value> = vec![];
                for a in attachments {
                    let attachment = self.upload_attachment(a).await?;
                    res.push(attachment);
                }
                res
            }
            None => vec![],
        };

        let timeout = timeout.unwrap_or(500);

        let payload =
            serde_json::json!({
             "completion": {
                "prompt": prompt,
                "timezone": "Asia/Saigon",
                "model": "claude-2"
            },
            "organization_uuid": self.org_uuid.clone(),
            "conversation_uuid": chat_uuid,
            "text": prompt,
            "attachments": attachments
            });

        let response = build_request(&self.cookies)?
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
            if data.get("completion").is_some() {
                completions.push(data["completion"].as_str().unwrap().to_string());
            }
        }

        let answer = completions.join("");

        debug!("response: {:#?}", answer);

        Ok(answer)
    }

    /// Renames a chat conversation.
    ///
    /// This function sends a POST request to the API to rename a chat conversation.
    ///
    /// # Arguments
    ///
    /// * `chat_uuid` - A string representing the UUID of the chat conversation to be renamed.
    /// * `title` - A string representing the new title for the chat conversation.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An empty `Result`, if the request is successful. Otherwise, an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the request fails.
    pub async fn rename_chat(&self, chat_uuid: &str, title: &str) -> Result<()> {
        let url = "https://claude.ai/api/rename_chat";

        let payload =
            serde_json::json!( {
            "organization_uuid": self.org_uuid.clone(),
            "conversation_uuid": chat_uuid.to_string(),
            "title": title.to_string(),
        });

        let res = build_request(&self.cookies)?.post(url).json(&payload).send().await?;

        debug!("response: {:#?}", res);

        Ok(())
    }
}
