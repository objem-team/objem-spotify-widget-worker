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
    router
        .get_async("/", routes::index::handler)
        .get("/login", routes::login::handler)
        .get_async("/callback", routes::callback::handler)
        .get_async("/next", routes::next::handler)
        .run(req, env)
        .await
}