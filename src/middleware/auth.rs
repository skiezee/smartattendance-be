use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use crate::utils::jwt;

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware { service }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        log::info!("🔐 JWT Auth Middleware - Path: {}", path);
        
        // Extract Authorization header
        let auth_header = req.headers().get("Authorization");
        
        if let Some(auth_value) = auth_header {
            log::info!("🔐 Authorization header found for path: {}", path);
            if let Ok(auth_str) = auth_value.to_str() {
                log::info!("🔐 Authorization value: {}", &auth_str[..20.min(auth_str.len())]);
                // Check if it starts with "Bearer "
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..]; // Remove "Bearer " prefix
                    log::info!("🔐 Token extracted, verifying for path: {}", path);
                    
                    // Verify token
                    match jwt::verify_token(token) {
                        Ok(claims) => {
                            log::info!("✅ JWT verification SUCCESS for path: {} - User: {}", path, claims.sub);
                            // Store claims in request extensions
                            req.extensions_mut().insert(claims);
                            
                            // Continue with the request
                            let fut = self.service.call(req);
                            return Box::pin(async move {
                                let res = fut.await?;
                                Ok(res)
                            });
                        }
                        Err(e) => {
                            log::error!("❌ JWT verification FAILED for path: {} - Error: {:?}", path, e);
                            return Box::pin(async move {
                                Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"))
                            });
                        }
                    }
                } else {
                    log::error!("❌ Authorization header doesn't start with 'Bearer ' for path: {}", path);
                }
            } else {
                log::error!("❌ Failed to parse Authorization header for path: {}", path);
            }
        } else {
            log::error!("❌ No Authorization header found for path: {}", path);
        }
        
        // No valid Authorization header found
        Box::pin(async move {
            Err(actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header"))
        })
    }
}
