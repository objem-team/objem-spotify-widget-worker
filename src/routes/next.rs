use http::StatusCode;
use rspotify::clients::OAuthClient;
use worker::{Request, Response, Result, RouteContext};

use crate::utils;

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let spotify = match utils::get_auth_code_spotify(&req, &ctx).await {
        Ok(spotify) => spotify,
        Err(error_response) => return error_response,
    };
    match spotify.next_track(None).await {
        Ok(_) => Response::ok(true.to_string()),
        Err(err) => Response::error(format!("{:?}", err), StatusCode::BAD_REQUEST.as_u16()),
    }
}
