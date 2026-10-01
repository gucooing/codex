//! First-party service routing for the ccodex distribution.
//!
//! The executable selects one service before loading credentials or starting clients.
//! Keep protocol identifiers and third-party destinations independent of this routing.

use reqwest::Url;
use std::io;
use std::sync::OnceLock;

pub const DEFAULT_BASE_OAUTH_URL: &str = "https://oauth-ai.alsl.xyz/api/oauth/chatgpt";
static SERVICE: OnceLock<ServiceEndpoint> = OnceLock::new();

#[derive(Debug, PartialEq, Eq)]
pub struct ServiceEndpoint {
    base: Url,
}

impl ServiceEndpoint {
    pub fn parse(value: &str) -> io::Result<Self> {
        let mut base = Url::parse(value)
            .map_err(|_| io::Error::other("BASE_OAUTH_URL must be an absolute HTTP(S) URL"))?;
        if !matches!(base.scheme(), "https" | "http")
            || base.host_str().is_none()
            || !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
            || value.trim() != value
        {
            return Err(io::Error::other(
                "BASE_OAUTH_URL must use HTTP(S) without credentials, query, fragment or surrounding whitespace",
            ));
        }
        let path = base.path().trim_end_matches('/').to_owned();
        base.set_path(&path);
        Ok(Self { base })
    }

    pub fn base_url(&self) -> &str {
        self.base.as_str().trim_end_matches('/')
    }

    pub fn route(&self, url: Url) -> Url {
        if !is_first_party(&url) {
            return url;
        }
        let mut routed = self.base.clone();
        routed.set_path(&format!(
            "{}{}",
            self.base.path().trim_end_matches('/'),
            url.path()
        ));
        routed.set_query(url.query());
        routed.set_fragment(url.fragment());
        if matches!(url.scheme(), "ws" | "wss") {
            let scheme = if self.base.scheme() == "http" {
                "ws"
            } else {
                "wss"
            };
            routed.set_scheme(scheme).expect("WebSocket scheme");
        }
        routed
    }
}

/// Selects an immutable service for this process; changing it requires a restart.
pub fn initialize_service(value: &str) -> io::Result<()> {
    let service = ServiceEndpoint::parse(value)?;
    if SERVICE.get().is_some_and(|current| current != &service) {
        return Err(io::Error::other(
            "Restart ccodex after changing BASE_OAUTH_URL",
        ));
    }
    let _ = SERVICE.set(service);
    Ok(())
}

pub fn active_service() -> Option<&'static ServiceEndpoint> {
    SERVICE.get()
}

pub fn route_service_url(url: Url) -> Url {
    match SERVICE.get() {
        Some(service) => service.route(url),
        None => url,
    }
}

pub fn service_url(value: &str) -> String {
    Url::parse(value)
        .map(route_service_url)
        .map(String::from)
        .unwrap_or_else(|_| value.to_owned())
}

/// A redirect back to an official service must never escape ccodex routing.
pub(crate) fn is_unrouted_service(url: &Url) -> bool {
    SERVICE.get().is_some() && is_first_party(url)
}

fn is_first_party(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http" | "ws" | "wss")
        && matches!(
            url.host_str(),
            Some(
                "auth.openai.com"
                    | "auth.api.openai.org"
                    | "api.openai.com"
                    | "chatgpt.com"
                    | "chat.openai.com"
                    | "chatgpt-staging.com"
                    | "api.chatgpt-staging.com"
                    | "ab.chatgpt.com"
                    | "platform.openai.com"
                    | "platform.api.openai.org"
            )
        )
}

#[cfg(test)]
#[path = "service_endpoint_tests.rs"]
mod tests;
