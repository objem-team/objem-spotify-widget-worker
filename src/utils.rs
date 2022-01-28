use std::collections::{HashMap, HashSet};

use cfg_if::cfg_if;
use http::{StatusCode};
use rspotify::{AuthCodeSpotify, Credentials, OAuth, Token};
use worker::{Error, Headers, Request, Response, RouteContext};

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn get_spotify_client(
    ctx: &RouteContext<()>,
) -> Result<AuthCodeSpotify, Box<dyn std::error::Error>> {
    let credentials = Credentials {
        id: ctx.secret("spotify_client_id")?.to_string(),
        secret: Some(ctx.secret("spotify_client_secret")?.to_string()),
    };
    let oauth = OAuth {
        redirect_uri: ctx.var("REDIRECT_URI")?.to_string(),
        scopes: ctx
            .var("SCOPES")?
            .to_string()
            .split(',')
            .map(|s| s.to_string())
            .collect::<HashSet<_>>(),
        ..Default::default()
    };
    Ok(AuthCodeSpotify::new(credentials, oauth))
}

pub fn get_spotify_credentials(
    ctx: &RouteContext<()>,
) -> Result<Credentials, Box<dyn std::error::Error>> {
    Ok(Credentials {
        id: ctx.secret("spotify_client_id")?.to_string(),
        secret: Some(ctx.secret("spotify_client_secret")?.to_string()),
    })
}

pub fn get_cookie_from_string(cookie_string: String) -> HashMap<String, String> {
    let mut cookie_map = HashMap::new();
    for cookie in cookie_string.split(';') {
        let mut cookie_parts = cookie.split('=');
        let key = cookie_parts.next().unwrap().trim().to_string();
        let value = cookie_parts.next().unwrap().trim().to_string();
        cookie_map.insert(key, value);
    }
    cookie_map
}

pub async fn get_auth_token_from_cookie(
    req: &Request,
    ctx: &RouteContext<()>,
) -> std::result::Result<Token, worker::Result<Response>> {
    let cookie_string = match req.headers().get("cookie") {
        Ok(cookie) => cookie,
        Err(err) => {
            return Err(Response::error(
                format!("UNAUTHORIZED : can't get cookie header \n {:?}", err),
                StatusCode::UNAUTHORIZED.as_u16(),
            ))
        }
    };
    let cookie_string = match cookie_string {
        Some(cookie) => cookie,
        None => {
            return Err(Response::error(
                "UNAUTHORIZED : cookie is None ",
                StatusCode::UNAUTHORIZED.as_u16(),
            ))
        }
    };
    let cookie = get_cookie_from_string(cookie_string);
    let session_id = match cookie.get("session_id") {
        Some(session_id) => session_id,
        None => {
            return Err(Response::error(
                "UNAUTHORIZED",
                StatusCode::UNAUTHORIZED.as_u16(),
            ))
        }
    };

    let kv = match ctx.kv("SESSION_KV") {
        Ok(kv) => kv,
        Err(err) => {
            return Err(Response::error(
                format!("INTERNAL_SERVER_ERROR : Can't get KvStore \n {:?}", err),
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ))
        }
    };

    let token_json = match kv.get(session_id).await {
        Ok(token_json) => token_json,
        Err(err) => {
            return Err(Response::error(
                format!(
                    "UNAUTHORIZED : can't get session value from KvStore \n {:?}",
                    err
                ),
                StatusCode::UNAUTHORIZED.as_u16(),
            ))
        }
    };

    let token_json = match token_json {
        Some(token_json) => token_json.as_string(),
        None => {
            return Err(Response::error(
                "UNAUTHORIZED : there is no session",
                StatusCode::UNAUTHORIZED.as_u16(),
            ))
        }
    };

    let token = match serde_json::from_str::<Token>(&token_json) {
        Ok(token) => token,
        Err(err) => {
            return Err(Response::error(
                format!("INTERNAL_SERVER_ERROR \n {:?}", err),
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ));
        }
    };
    Ok(token)
}

pub async fn get_auth_code_spotify(
    req: &Request,
    ctx: &RouteContext<()>,
) -> std::result::Result<AuthCodeSpotify, worker::Result<Response>> {
    let token = match get_auth_token_from_cookie(req, ctx).await {
        Ok(token) => token,
        Err(err) => return Err(err),
    };
    let mut spotify = AuthCodeSpotify::from_token(token);
    spotify.creds = get_spotify_credentials(&ctx).unwrap();
    spotify.config.token_refreshing = true;
    Ok(spotify)
}

pub fn append_cors_header(headers: &mut Headers) -> std::result::Result<(), Error> {
    headers.append("Access-Control-Allow-Origin", "https://objem.app")?;
    headers.append("access-control-allow-credentials", "true")?;
    headers.append("access-control-allow-methods", "GET")?;
    Ok(())
}