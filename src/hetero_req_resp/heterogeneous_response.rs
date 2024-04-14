use std::convert::Infallible;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{HeaderValue};
use axum::http::header::ACCEPT;
use axum::http::request::Parts;

pub enum AcceptType {
    HTMX,
    JSON
}

#[async_trait]
impl<S> FromRequestParts<S> for AcceptType
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let accept_header = parts.headers.get(ACCEPT);
        let accept_type = characterize_accept_type(accept_header);
        Ok(accept_type.unwrap_or(AcceptType::HTMX))
    }
}

fn characterize_accept_type(accept: Option<&HeaderValue>) -> Option<AcceptType> {
    use AcceptType::*;
    let Some(accept) = accept else {
        return None
    };
    let Ok(accept_str) = accept.to_str() else {
        eprintln!(
            "accept header was not ascii! {:?}",
            accept
        );
        return None;
    };

    let html_accept_index = accept_str.find("text/html");
    let json_accept_index = accept_str.find("application/json");

    match (html_accept_index, json_accept_index) {
        (Some(_), None) => Some(HTMX),
        (None, Some(_)) => Some(JSON),
        (Some(html), Some(json)) if html < json => Some(HTMX),
        (Some(html), Some(json)) if json < html => Some(JSON),
        _ => None
    }
}