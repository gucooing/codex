use super::*;
use pretty_assertions::assert_eq;

#[test]
fn preserves_paths_queries_and_websocket_transport() {
    let endpoint = ServiceEndpoint::parse("http://127.0.0.1:8080/api/oauth/chatgpt///").unwrap();
    for (source, expected) in [
        (
            "https://auth.openai.com/oauth/token",
            "http://127.0.0.1:8080/api/oauth/chatgpt/oauth/token",
        ),
        (
            "https://chatgpt.com/backend-api/codex/models?client_version=0.159.3",
            "http://127.0.0.1:8080/api/oauth/chatgpt/backend-api/codex/models?client_version=0.159.3",
        ),
        (
            "wss://chatgpt.com/backend-api/codex/responses?x=a%2Fb",
            "ws://127.0.0.1:8080/api/oauth/chatgpt/backend-api/codex/responses?x=a%2Fb",
        ),
        (
            "https://api.openai.com/v1/responses/input_tokens",
            "http://127.0.0.1:8080/api/oauth/chatgpt/v1/responses/input_tokens",
        ),
        (
            "https://auth.openai.com/oauth/authorize?state=s&code_challenge=c&redirect_uri=http%3A%2F%2F127.0.0.1%3A1455%2Fauth%2Fcallback",
            "http://127.0.0.1:8080/api/oauth/chatgpt/oauth/authorize?state=s&code_challenge=c&redirect_uri=http%3A%2F%2F127.0.0.1%3A1455%2Fauth%2Fcallback",
        ),
    ] {
        assert_eq!(
            endpoint.route(Url::parse(source).unwrap()).as_str(),
            expected
        );
    }
}

#[test]
fn routing_is_idempotent_and_does_not_rewrite_third_party_or_similar_hosts() {
    let endpoint = ServiceEndpoint::parse(DEFAULT_BASE_OAUTH_URL).unwrap();
    for value in [
        "https://example.com/mcp",
        "https://chatgpt.com.evil.example/backend-api",
        "https://github.com/gucooing/codex/releases",
        "https://oauth-ai.alsl.xyz/api/oauth/chatgpt/oauth/token",
    ] {
        let url = Url::parse(value).unwrap();
        assert_eq!(endpoint.route(url.clone()), url);
    }
}

#[test]
fn invalid_service_configuration_fails() {
    for value in [
        "",
        "/relative",
        "ftp://example.com",
        "https://u:p@example.com",
        "https://example.com?q=1",
        "https://example.com#fragment",
        " https://example.com",
    ] {
        assert!(ServiceEndpoint::parse(value).is_err(), "{value}");
    }
}
