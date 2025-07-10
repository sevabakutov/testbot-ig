use std::env;
use once_cell::sync::Lazy;

pub static IG_USER_ID: Lazy<String> = Lazy::new(|| env::var("IG_USER_ID").expect("IG_USER_ID env-var is missing"));

pub static ACCESS_TOKEN: Lazy<String> = Lazy::new(|| env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN env-var is missing"));

pub static VERIFY_TOKEN: Lazy<String> = Lazy::new(|| env::var("TOKEN").expect("TOKEN env-var is missing"));

pub static APP_SECRET: Lazy<String> = Lazy::new(|| env::var("APP_SECRET").expect("APP_SECRT env-var is missing"));

// pub static OPENAI_API_KEY: Lazy<String> = Lazy::new(|| env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY env-var is missing"));

pub static OPENAI_PROJECT_ID: Lazy<String> = Lazy::new(|| env::var("OPENAI_PROJECT_ID").expect("OPENAI_PROJECT_ID env-var is missing"));

pub const DM_URL: &str = "https://graph.instagram.com/v21.0/me/messages";