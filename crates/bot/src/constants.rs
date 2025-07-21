use std::env;
use once_cell::sync::Lazy;

/// инстаграм айди бота
pub static IG_USER_ID: Lazy<String> = Lazy::new(|| env::var("IG_USER_ID").expect("IG_USER_ID env-var is missing"));
pub static ACCESS_TOKEN: Lazy<String> = Lazy::new(|| env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN env-var is missing"));
pub static VERIFY_TOKEN: Lazy<String> = Lazy::new(|| env::var("TOKEN").expect("TOKEN env-var is missing"));
pub static APP_SECRET: Lazy<String> = Lazy::new(|| env::var("APP_SECRET").expect("APP_SECRT env-var is missing"));
pub static OPENAI_PROJECT_ID: Lazy<String> = Lazy::new(|| env::var("OPENAI_PROJECT_ID").expect("OPENAI_PROJECT_ID env-var is missing"));
pub static QDRANT_API_KEY: Lazy<String> = Lazy::new(|| env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY env-var is missing")); 

pub const DM_URL: &str = "https://graph.instagram.com/v21.0/me/messages";
pub const QDRANT_URL: &str = "https://288c8d6c-d456-4a3a-8895-e97b194bbf2a.eu-west-2-0.aws.cloud.qdrant.io:6334";

/// Размер вектора
pub const DIM_SIZE: u64 = 1536;

/// Единственная и основная коллекция в векторной базе данных
pub const QDRANT_COLLECTION_NAME: &'static str = "answers";

/// Сколько ждать после последнего входящего символа, прежде чем "слить" буфер.
pub const WINDOW_MS: u64 = 4500;

/// Символьный лимит сообщения
pub const IG_LIMIT: usize = 950;
