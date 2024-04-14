use std::fmt::Debug;
use askama_axum::{IntoResponse, Response};
use axum::{async_trait, Form, Json};
use axum::extract::{FromRequest, Request};
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use serde::de::DeserializeOwned;

const FORM_HEADER: &str = "application/x-www-form-urlencoded";
const JSON_HEADER: &str = "application/json";

pub struct JsonOrForm<T>(pub T);

#[derive(thiserror::Error, Debug)]
pub enum JsonOrFormRejection{
    #[error("json parse error `{0}`")]
    Json(#[from] JsonRejection),
    #[error("form parse error `{0}`")]
    Form(#[from] FormRejection),
    #[error("No content type matched")]
    None
}

impl IntoResponse for JsonOrFormRejection{
    fn into_response(self) -> Response {
        match self {
            JsonOrFormRejection::Json(j) => j.into_response(),
            JsonOrFormRejection::Form(f) => f.into_response(),
            JsonOrFormRejection::None => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                format!("Expected request with `Content-Type: {}` or `Content-Type: {}`", FORM_HEADER, JSON_HEADER)
            ).into_response()
        }
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for JsonOrForm<T>
    where S: Send + Sync,
          T: DeserializeOwned,
{
    type Rejection = JsonOrFormRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type_header = req.headers().get(CONTENT_TYPE);
        let content_type = characterize_content_type(content_type_header)
            .ok_or(JsonOrFormRejection::None)?;

        let result: T = match content_type {
            ContentType::JSON => Json::from_request(req, state).await?.0,
            ContentType::FORM => Form::from_request(req, state).await?.0,
        };

        Ok(Self(result))
    }
}

#[derive(Debug)]
enum ContentType {
    FORM,
    JSON
}

fn characterize_content_type(content: Option<&HeaderValue>) -> Option<ContentType> {
    let Some(content) = content else {
        return None
    };
    let Ok(content_str) = content.to_str() else {
        eprintln!(
            "content header was not ascii! {:?}",
            content
        );
        return None;
    };

    let form_index = content_str.find(FORM_HEADER);
    let json_index = content_str.find(JSON_HEADER);

    match (form_index, json_index) {
        (Some(_), None) => Some(ContentType::FORM),
        (None, Some(_)) => Some(ContentType::JSON),
        (Some(html), Some(json)) if html < json => Some(ContentType::FORM),
        (Some(html), Some(json)) if json <= html => Some(ContentType::JSON),
        _ => None
    }
}