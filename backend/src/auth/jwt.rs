//! JWT creation and validation.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user id
    pub exp: i64,
    pub iat: i64,
    pub email: String,
}

pub fn create_token(user_id: Uuid, email: &str, secret: &str, expiry_hours: i64) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + Duration::hours(expiry_hours);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        email: email.to_string(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode failed: {}", e)))?;
    Ok(token)
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Auth("Invalid or expired token".to_string()))?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_create_and_validate_token() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let secret = "test_secret_at_least_32_chars_long";
        let token = create_token(user_id, email, secret, 1).unwrap();
        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
    }
}
