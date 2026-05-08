use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // Subject (NIK)
    pub name: String,     // Employee name
    pub role: String,     // Employee role
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
}

impl Claims {
    pub fn new(nik: String, name: String, role: String) -> Self {
        let iat = Utc::now();
        let exp_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<i64>()
            .unwrap_or(24);
        let exp = iat + Duration::hours(exp_hours);

        Claims {
            sub: nik,
            name,
            role,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

pub fn generate_token(nik: String, name: String, role: String) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(nik, name, role);
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}

// Helper function for alternative authentication approach
#[allow(dead_code)]
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    
    match verify_token(token) {
        Ok(claims) => {
            // Store claims in request extensions for later use
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(e) => {
            log::error!("JWT validation failed: {:?}", e);
            Err((actix_web::error::ErrorUnauthorized("Invalid token"), req))
        }
    }
}

// Helper function to extract claims from request
#[allow(dead_code)]
pub fn get_claims_from_request(req: &actix_web::HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}
