use std::string::FromUtf8Error;
use std::time::SystemTime;
use secstr::SecStr;
use serde::{Deserialize, Deserializer};
use serde_json::Number;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// By default "bearer" is the token type that the concourse API will return since we are using
    /// the oauth2 flow. In the Concourse source code "bearer" appears as the default value.
    Bearer,

    /// This a catch all for any token type that we don't know about. This will not occur here since
    /// all the endpoints we are using in this implementation accept a bearer token.
    Other(String),
}

impl<'de> Deserialize<'de> for TokenType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "bearer" => TokenType::Bearer,
            _ => TokenType::Other(s),
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Token {
    /// The token that authorizes and authenticates the HTTP requests to the Concourse API.
    pub access_token: SecStr,
    #[serde(deserialize_with = "deserialize_to_duration")]
    pub expires_in: std::time::Duration,
    pub id_token: String,
    pub token_type: TokenType,
    #[serde(default = "SystemTime::now")]
    pub created_at: SystemTime,
}

impl Token {
    pub fn get_access_token(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.access_token.unsecure().to_vec())
    }

    pub fn has_expired(&self) -> bool {
        let now = SystemTime::now();
        let expires_at = self.created_at + self.expires_in;
        now > expires_at
    }
}

pub fn deserialize_to_duration<'de, D>(deserializer: D) -> Result<std::time::Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
{
    let n = Number::deserialize(deserializer)?;
    let secs = n.as_u64().map_or(Err(serde::de::Error::custom("not a u64")), Ok)?;
    Ok(std::time::Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};
    use crate::concourse::token::{Token, TokenType};

    #[test]
    fn will_successfully_deserialize_to_token_struct() -> Result<(), serde_json::Error> {
        let string = r#"
            {
                "access_token": "token",
                "expires_in": 123456,
                "id_token": "token-id",
                "token_type": "bearer"
            }
        "#;

        let token: Token = serde_json::from_str(string)?;

        assert_eq!(token.access_token, secstr::SecStr::from("token"));
        assert_eq!(token.expires_in, std::time::Duration::from_secs(123456));
        assert_eq!(token.id_token, String::from("token-id"));
        assert_eq!(token.token_type, TokenType::Bearer);
        Ok(())
    }

    #[test]
    fn will_successfully_deserialize_to_token_struct_selecting_other_for_token_type() -> Result<(), serde_json::Error> {
        let string = r#"
            {
                "access_token": "token",
                "expires_in": 123456,
                "id_token": "token-id",
                "token_type": "something-else"
            }
        "#;

        let token: Token = serde_json::from_str(string)?;

        assert_eq!(token.token_type, TokenType::Other(String::from("something-else")));
        Ok(())
    }

    #[test]
    fn will_return_an_error_if_expires_in_is_a_string() -> Result<(), serde_json::Error> {
        let string = r#"
            {
                "access_token": "token",
                "expires_in": "123456",
                "id_token": "token-id",
                "token_type": "bearer"
            }
        "#;

        let result = serde_json::from_str::<Token>(string);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn will_return_an_error_if_the_expires_in_is_not_a_u64() -> Result<(), serde_json::Error> {
        let string = r#"
            {
                "access_token": "token",
                "expires_in": 123456.789,
                "id_token": "token-id",
                "token_type": "bearer"
            }
        "#;

        let result = serde_json::from_str::<Token>(string);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn will_not_expire_when_current_time_is_before_expires_in_field() {
        let token = Token {
            access_token: "access-token".parse().unwrap(),
            expires_in: Duration::from_secs(10),
            id_token: "id-token".to_string(),
            token_type: TokenType::Bearer,
            created_at: SystemTime::now(),
        };

        assert!(!token.has_expired());
    }

    #[test]
    fn will_expire_when_current_time_is_after_expires_in_field() {
        let token = Token {
            access_token: "access-token".parse().unwrap(),
            expires_in: Duration::from_secs(1),
            id_token: "id-token".to_string(),
            token_type: TokenType::Bearer,
            created_at: SystemTime::now() - Duration::from_secs(2),
        };

        assert!(token.has_expired());
    }
}
