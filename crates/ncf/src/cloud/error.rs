#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudError {
    Offline,
    SignedOut,
    Forbidden,
    NotFound,
    Unconfirmed,
    Rate,
    Reauth,
    Invalid(String),
    Limit(String),
    Auth(String),
    Server(String),
}

impl CloudError {
    pub fn from_code(message: &str) -> CloudError {
        let m = message.trim();
        let Some(code) = m.strip_prefix("nc:") else {
            return CloudError::Server(m.to_string());
        };
        let (head, what) = code.split_once(':').unwrap_or((code, ""));
        match head {
            "auth" => CloudError::SignedOut,
            "forbidden" => CloudError::Forbidden,
            "not_found" => CloudError::NotFound,
            "unconfirmed" => CloudError::Unconfirmed,
            "rate" => CloudError::Rate,
            "reauth" => CloudError::Reauth,
            "invalid" => CloudError::Invalid(what.to_string()),
            "limit" => CloudError::Limit(what.to_string()),
            _ => CloudError::Server(m.to_string()),
        }
    }

    pub fn from_auth(code: &str, message: &str) -> CloudError {
        match code {
            "email_not_confirmed" => CloudError::Unconfirmed,
            "over_email_send_rate_limit" | "over_request_rate_limit" | "over_sms_send_rate_limit" => CloudError::Rate,
            "reauthentication_needed" | "reauthentication_not_valid" => CloudError::Reauth,
            "refresh_token_not_found" | "refresh_token_already_used" | "session_not_found" | "session_expired" | "bad_jwt" | "no_authorization" => {
                CloudError::SignedOut
            }
            "" => CloudError::Auth(message.to_string()),
            other => CloudError::Auth(other.to_string()),
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            CloudError::Offline => "You're offline",
            CloudError::SignedOut => "Signed out",
            CloudError::Forbidden => "Not allowed",
            CloudError::NotFound => "Not found",
            CloudError::Unconfirmed => "Confirm your email first",
            CloudError::Rate => "Slow down a moment",
            CloudError::Reauth => "Sign in again",
            CloudError::Invalid(_) => "Check that",
            CloudError::Limit(_) => "Plan limit reached",
            CloudError::Auth(_) => "Couldn't sign in",
            CloudError::Server(_) => "NativeCloud had a problem",
        }
    }

    pub fn message(&self) -> String {
        match self {
            CloudError::Offline => "NativeCloud can't be reached. Your work is kept on this computer and syncs when you're back.".into(),
            CloudError::SignedOut => "Your session ended. Sign in again to keep syncing.".into(),
            CloudError::Forbidden => "Your role in this team doesn't allow that.".into(),
            CloudError::NotFound => "It may have been deleted, or you no longer have access.".into(),
            CloudError::Unconfirmed => "Enter the 6-digit code we emailed you.".into(),
            CloudError::Rate => "That was a lot at once. Wait a minute and try again.".into(),
            CloudError::Reauth => "For your safety, enter your password again first.".into(),
            CloudError::Invalid(what) => match what.as_str() {
                "name" => "Names need 1 to 80 characters.".into(),
                "email" => "That doesn't look like an email address.".into(),
                "role" => "Pick admin, editor or viewer.".into(),
                "owner" => "The owner can't do that. Hand the team over first.".into(),
                "member" => "That person isn't in this team.".into(),
                "personal" => "That can't be done with My cloud.".into(),
                "not_deleted" => "Move it to Recently deleted first.".into(),
                other => format!("NativeCloud didn't accept that ({other})."),
            },
            CloudError::Limit(what) => match what.as_str() {
                "projects" => "You've reached your plan's number of cloud scripts. Move some to this computer or delete old ones.".into(),
                "storage" => "You've used your plan's storage. Empty Recently deleted or remove old scripts.".into(),
                "teams" => "You own as many teams as your plan allows.".into(),
                "members" => "This team has as many people as its plan allows.".into(),
                "snapshots" => "This script has as many named snapshots as your plan allows. Delete an old one first.".into(),
                other => format!("Your plan's limit for {other} is reached."),
            },
            CloudError::Auth(code) => match code.as_str() {
                "invalid_credentials" | "invalid_grant" => "That email and password don't match.".into(),
                "weak_password" => "Use at least 10 characters, with upper and lower case letters and a digit.".into(),
                "otp_expired" | "otp_disabled" => "That code is wrong or has expired. Send a new one.".into(),
                "user_already_exists" | "email_exists" => "There's already an account with that email. Sign in instead.".into(),
                "same_password" => "Choose a password you haven't used here before.".into(),
                "email_address_invalid" | "validation_failed" => "Check the email address.".into(),
                "signup_disabled" => "New accounts are closed for now.".into(),
                "user_banned" => "This account is suspended. Contact hello@nativecompute.com.".into(),
                other => other.to_string(),
            },
            CloudError::Server(m) => {
                if m.is_empty() {
                    "Try again in a moment.".into()
                } else {
                    format!("Try again in a moment. ({m})")
                }
            }
        }
    }

    pub fn is_offline(&self) -> bool {
        matches!(self, CloudError::Offline)
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, CloudError::Offline | CloudError::Rate | CloudError::Server(_))
    }
}

impl std::fmt::Display for CloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title(), self.message())
    }
}

impl std::error::Error for CloudError {}
