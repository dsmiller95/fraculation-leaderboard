use axum::http::HeaderValue;

pub enum ResponseType{
    HTMX,
    JSON
}

pub fn characterize_accept_type(accept: Option<&HeaderValue>) -> Option<ResponseType> {
    use ResponseType::*;
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

    println!("accept header found indexes: html: {:?} json: {:?}", html_accept_index, json_accept_index);

    match (html_accept_index, json_accept_index) {
        (Some(_), None) => Some(HTMX),
        (None, Some(_)) => Some(JSON),
        (Some(html), Some(json)) if html < json => Some(HTMX),
        (Some(html), Some(json)) if json < html => Some(JSON),
        _ => None
    }
}