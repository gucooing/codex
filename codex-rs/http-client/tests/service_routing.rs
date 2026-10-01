use codex_http_client::HttpClient;
use codex_http_client::service_endpoint::initialize_service;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn first_party_token_requests_reach_configured_service_with_unchanged_body() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    initialize_service(&format!("http://{address}/api/oauth/chatgpt")).unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut buffer = [0; 4096];
            let count = socket.read(&mut buffer).await.unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
            if request.ends_with(b"fixture-refresh") {
                break;
            }
        }
        let request = String::from_utf8(request).unwrap();
        assert!(request.starts_with("POST /api/oauth/chatgpt/oauth/token HTTP/1.1\r\n"));
        assert!(request.ends_with("grant_type=refresh_token&refresh_token=fixture-refresh"));
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .await
            .unwrap();
    });
    let response = HttpClient::new(reqwest::Client::new())
        .post("https://auth.openai.com/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("grant_type=refresh_token&refresh_token=fixture-refresh")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    server.await.unwrap();
}
