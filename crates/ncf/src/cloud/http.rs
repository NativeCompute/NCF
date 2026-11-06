use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl Request {
    pub fn new(method: &'static str, url: String) -> Request {
        Request { method, url, headers: Vec::new(), body: None }
    }

    pub fn header(mut self, k: &str, v: &str) -> Request {
        self.headers.push((k.to_string(), v.to_string()));
        self
    }

    pub fn json(mut self, body: &serde_json::Value) -> Request {
        self.body = Some(body.to_string());
        self.header("content-type", "application/json")
    }

    pub fn get_header(&self, k: &str) -> Option<&str> {
        self.headers.iter().find(|(h, _)| h.eq_ignore_ascii_case(k)).map(|(_, v)| v.as_str())
    }

    pub fn path(&self) -> &str {
        match self.url.find("://") {
            Some(i) => {
                let rest = &self.url[i + 3..];
                rest.find('/').map(|j| &rest[j..]).unwrap_or("/")
            }
            None => &self.url,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub date: Option<String>,
}

impl Reply {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn json(&self) -> serde_json::Value {
        if self.body.trim().is_empty() {
            return serde_json::Value::Null;
        }
        serde_json::from_str(&self.body).unwrap_or(serde_json::Value::Null)
    }
}

pub trait Transport: Send + Sync {
    fn send(&self, req: &Request) -> Result<Reply, String>;
}

pub struct Ureq {
    agent: ureq::Agent,
}

impl Ureq {
    pub fn new(user_agent: &str) -> Ureq {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(8))
            .timeout(Duration::from_secs(30))
            .user_agent(user_agent)
            .build();
        Ureq { agent }
    }
}

impl Transport for Ureq {
    fn send(&self, req: &Request) -> Result<Reply, String> {
        let mut r = self.agent.request(req.method, &req.url);
        for (k, v) in &req.headers {
            r = r.set(k, v);
        }
        let result = match &req.body {
            Some(b) => r.send_string(b),
            None => r.call(),
        };
        let resp = match result {
            Ok(resp) => resp,
            Err(ureq::Error::Status(_, resp)) => resp,
            Err(e) => return Err(e.to_string()),
        };
        let status = resp.status();
        let date = resp.header("date").map(str::to_string);
        let body = resp.into_string().map_err(|e| e.to_string())?;
        Ok(Reply { status, body, date })
    }
}
