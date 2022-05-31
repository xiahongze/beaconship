use hyper::{header, Method, Request, StatusCode};
use serde::Serialize;

use crate::client::ClientType;

const PUSHOVER_URL: &str = "https://api.pushover.net/1/messages.json";

#[derive(Serialize)]
struct PushOverMsg<'a> {
    message: &'a str,
    token: &'a str,
    user: &'a str,
}

pub async fn send_notice(msg: &str, app_token: &str, user_token: &str, client: &ClientType) {
    let body = serde_json::to_vec(&PushOverMsg {
        message: msg,
        token: app_token,
        user: user_token,
    })
    .unwrap();
    let req = Request::builder()
        .uri(PUSHOVER_URL)
        .header(header::CONTENT_TYPE, "application/json")
        .method(Method::POST)
        .body(body.into())
        .unwrap();
    let result = client.request(req).await;
    match result {
        Ok(resp) => match resp.status() {
            StatusCode::OK => info!("pushover message sent"),
            code => warn!("failed with statusCode {:?}, msg {:?}", code, *resp.body()),
        },
        Err(err) => warn!("request sent failed with error, {:?}", err),
    }
}
