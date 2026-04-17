use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use yup_oauth2::ServiceAccountAuthenticator;
use log::{info, error};

const SCOPES: [&str; 1] = ["https://www.googleapis.com/auth/firebase.messaging"];

pub async fn send_fcm_notification(
    project_id: &str,
    token: &str,
    title: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let secret = if let Ok(json_content) = std::env::var("FIREBASE_SERVICE_ACCOUNT_JSON") {
        serde_json::from_str(&json_content)?
    } else {
        let credentials_path = std::env::var("FIREBASE_CREDENTIALS_PATH").unwrap_or_else(|_| "config/firebase-service-account.json".to_string());
        yup_oauth2::read_service_account_key(credentials_path).await?
    };
    
    let authenticator = ServiceAccountAuthenticator::builder(secret).build().await?;
    
    let token_response = authenticator.token(&SCOPES).await?;
    let access_token = token_response.token().unwrap();

    let url = format!(
        "https://fcm.googleapis.com/v1/projects/{}/messages:send",
        project_id
    );

    let client = reqwest::Client::new();
    let payload = json!({
        "message": {
            "token": token,
            "notification": {
                "title": title,
                "body": body
            }
        }
    });

    let res = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        info!("Successfully sent FCM notification to {}", token);
    } else {
        let error_text = res.text().await?;
        error!("Failed to send FCM notification: {}", error_text);
    }

    Ok(())
}
