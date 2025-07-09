// use std::env;

// use actix_web::Result;

// use crate::models::{Message, Sender};

// pub async fn send_message(sender: Sender, message: Message) -> Result<()> {
//     let ig_user_id = env::var("IG_USER_ID")?;
//     let token      = env::var("IG_PAGE_TOKEN")?;   // Page Access Token

//     let url = format!(
//         "https://graph.facebook.com/v23.0/{ig_user_id}/messages"
//     );

//     let body = SendBody {
//         recipient: Recipient { id: sender_id },
//         message:   Message   { text: GREETING },
//     };

//     http.post(url)
//         .query(&[("access_token", token)])
//         .json(&body)
//         .send()
//         .await?
//         .error_for_status()?;                       // выбросит, если HTTP != 2xx
//     Ok(())
// }