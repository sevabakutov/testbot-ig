use std::env;

use actix_web::{HttpRequest, HttpResponse};

pub fn verify_challenge(req: &HttpRequest) -> HttpResponse {
    let verify_token = match env::var("TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("❌ TOKEN env var missing");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let qs = req.query_string();

    let mut mode = None;
    let mut token = None;
    let mut challenge = None;

    for (key, value) in url::form_urlencoded::parse(qs.as_bytes()) {
        match key.as_ref() {
            "hub.mode"         => mode      = Some(value.into_owned()),
            "hub.verify_token" => token     = Some(value.into_owned()),
            "hub.challenge"    => challenge = Some(value.into_owned()),
            _ => {}
        }
    }

    if mode.as_deref() == Some("subscribe") && token.as_deref() == Some(&verify_token) {
        println!("✅ Webhook verified");
        HttpResponse::Ok().body(challenge.unwrap_or_default())
    } else {
        println!("❌ Verification failed");
        HttpResponse::Forbidden().finish()
    }
}

pub fn verify_signature(req: &HttpRequest, body: &[u8]) -> Result<(), HttpResponse> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let app_secret = env::var("APP_SECRET").map_err(|_| {
        eprintln!("❌ APP_SECRET not set");
        HttpResponse::InternalServerError().finish()
    })?;

    let signature_header = req
        .headers()
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("sha256="))
        .ok_or_else(|| {
            eprintln!("❌ Invalid or missing X-Hub-Signature-256");
            HttpResponse::Unauthorized().finish()
        })?;

    let expected_sig = hex::decode(signature_header).map_err(|_| {
        eprintln!("❌ Signature hex decoding failed");
        HttpResponse::Unauthorized().finish()
    })?;

    let mut mac = HmacSha256::new_from_slice(app_secret.as_bytes()).map_err(|_| {
        eprintln!("❌ HMAC key");
        HttpResponse::Unauthorized().finish()
    })?;
    mac.update(body);

    mac.verify_slice(&expected_sig).map_err(|_| {
        eprintln!("❌ Signature mismatch");
        HttpResponse::Unauthorized().finish()
    })?;

    println!("🔒 Signature verified");
    Ok(())
}