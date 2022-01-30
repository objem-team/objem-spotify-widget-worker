use http::{StatusCode};
use worker::*;
mod routes;
mod utils;
fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();
    let router = Router::new();
    let response = router
        .options( "/*route", routes::preflight::handler)
        .get_async("/", routes::index::handler)
        .get("/login", routes::login::handler)
        .get_async("/callback", routes::callback::handler)
        .put_async("/next", routes::next::handler)
        .put_async("/previous", routes::previous::handler)
        .put_async("/pause", routes::pause::handler)
        .put_async("/resume", routes::resume_playback::handler)
        .put_async("/shuffle", routes::shuffle::handler)
        .put_async("/repeat",routes::repeat::handler)
        .run(req, env)
        .await;
    let mut response = match response {
        Ok(response) => response,
        Err(err) => return Err(err),
    };
    if response.status_code() == StatusCode::FOUND.as_u16() {
        return Ok(response);
    }
    let headers = response.headers_mut();
    utils::append_cors_header(headers)?;
    Ok(response)
}
