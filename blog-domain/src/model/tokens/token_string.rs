pub trait TokenString {
    fn str(&self) -> &str;
}

pub struct AccessTokenString(pub String);

impl TokenString for AccessTokenString {
    fn str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AccessTokenString {
    fn from(token: String) -> Self {
        AccessTokenString(token)
    }
}

pub struct IdTokenString(pub String);

impl TokenString for IdTokenString {
    fn str(&self) -> &str {
        &self.0
    }
}

impl From<String> for IdTokenString {
    fn from(token: String) -> Self {
        IdTokenString(token)
    }
}

pub struct RefreshTokenString(pub String);

impl TokenString for RefreshTokenString {
    fn str(&self) -> &str {
        &self.0
    }
}
