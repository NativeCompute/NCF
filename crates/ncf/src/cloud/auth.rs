use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{Client, CloudError};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub user_id: String,
    pub email: String,
}

impl Session {
    pub fn from_value(v: &Value, now: i64) -> Option<Session> {
        let access_token = v.get("access_token")?.as_str()?.to_string();
        let refresh_token = v.get("refresh_token")?.as_str()?.to_string();
        let expires_at = v
            .get("expires_at")
            .and_then(Value::as_i64)
            .or_else(|| v.get("expires_in").and_then(Value::as_i64).map(|s| now + s))
            .unwrap_or(now + 3600);
        let user = v.get("user").cloned().unwrap_or(Value::Null);
        Some(Session {
            access_token,
            refresh_token,
            expires_at,
            user_id: user.get("id").and_then(Value::as_str).unwrap_or("").to_string(),
            email: user.get("email").and_then(Value::as_str).unwrap_or("").to_string(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignUp {
    NeedsCode,
    SignedIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    SignUp,
    Recovery,
    EmailChange,
}

impl Code {
    pub fn kind(self) -> &'static str {
        match self {
            Code::SignUp => "signup",
            Code::Recovery => "recovery",
            Code::EmailChange => "email_change",
        }
    }
}

impl Client {
    pub fn sign_up(&self, email: &str, password: &str, name: &str) -> Result<SignUp, CloudError> {
        let body = json!({"email": email.trim().to_lowercase(), "password": password, "data": {"display_name": name.trim()}});
        let v = self.auth_post("/auth/v1/signup", &body)?;
        match Session::from_value(&v, self.server_now()) {
            Some(s) => {
                self.set_session(Some(s));
                Ok(SignUp::SignedIn)
            }
            None => Ok(SignUp::NeedsCode),
        }
    }

    pub fn verify(&self, code: Code, email: &str, token: &str) -> Result<Session, CloudError> {
        let token: String = token.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        let body = json!({"type": code.kind(), "email": email.trim().to_lowercase(), "token": token});
        let v = self.auth_post("/auth/v1/verify", &body)?;
        let s = Session::from_value(&v, self.server_now()).ok_or_else(|| CloudError::Server("no session in the reply".into()))?;
        self.set_session(Some(s.clone()));
        Ok(s)
    }

    pub fn resend(&self, code: Code, email: &str) -> Result<(), CloudError> {
        let body = json!({"type": code.kind(), "email": email.trim().to_lowercase()});
        self.auth_post("/auth/v1/resend", &body).map(|_| ())
    }

    pub fn sign_in(&self, email: &str, password: &str) -> Result<Session, CloudError> {
        let body = json!({"email": email.trim().to_lowercase(), "password": password});
        let v = self.auth_post("/auth/v1/token?grant_type=password", &body)?;
        let s = Session::from_value(&v, self.server_now()).ok_or_else(|| CloudError::Server("no session in the reply".into()))?;
        self.set_session(Some(s.clone()));
        Ok(s)
    }

    pub fn recover(&self, email: &str) -> Result<(), CloudError> {
        self.auth_post("/auth/v1/recover", &json!({"email": email.trim().to_lowercase()})).map(|_| ())
    }

    pub fn set_password(&self, password: &str) -> Result<(), CloudError> {
        self.auth_user(&json!({"password": password})).map(|_| ())
    }

    pub fn change_email(&self, email: &str) -> Result<(), CloudError> {
        self.auth_user(&json!({"email": email.trim().to_lowercase()})).map(|_| ())
    }

    pub fn sign_out(&self, everywhere: bool) -> Result<(), CloudError> {
        let scope = if everywhere { "global" } else { "local" };
        let path = format!("/auth/v1/logout?scope={scope}");
        let result = if self.signed_in() { self.authed_send("POST", &path, Some(&json!({}))).map(|_| ()) } else { Ok(()) };
        self.set_session(None);
        match result {
            Err(CloudError::SignedOut) => Ok(()),
            other => other,
        }
    }

    pub fn delete_account(&self) -> Result<(), CloudError> {
        self.function("delete-account", &json!({}))?;
        self.set_session(None);
        Ok(())
    }

    pub fn reauthenticate(&self, password: &str) -> Result<(), CloudError> {
        let email = self.session().map(|s| s.email).unwrap_or_default();
        if email.is_empty() {
            return Err(CloudError::SignedOut);
        }
        self.sign_in(&email, password).map(|_| ())
    }
}
