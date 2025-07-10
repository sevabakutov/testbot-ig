use std::{collections::HashMap, env, sync::Mutex};
use once_cell::sync::Lazy;

pub static ESCALATED_CHATS: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub static IG_USER_ID: Lazy<String> = Lazy::new(|| {
    env::var("IG_USER_ID").expect("IG_USER_ID env-var is missing")
});

pub static ACCESS_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN env-var is missing")
});

pub static VERIFY_TOKEN: Lazy<String> = Lazy::new(|| {
    env::var("TOKEN").expect("TOKEN env-var is missing")
});

pub static APP_SECRET: Lazy<String> = Lazy::new(|| {
    env::var("APP_SECRET").expect("APP_SECRT env-var is missing")
});

pub const DM_URL: &str = "https://graph.instagram.com/v21.0/me/messages";