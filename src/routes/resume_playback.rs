use http::StatusCode;
use rspotify::{clients::OAuthClient, ClientError};
use worker::{Request, Response, Result, RouteContext};

use crate::utils;

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let spotify = match utils::get_auth_code_spotify(&req, &ctx).await {
        Ok(spotify) => spotify,
        Err(error_response) => return error_response,
    };
    let err = match spotify.resume_playback(None,None).await {
        Ok(_) => return Response::ok(true.to_string()),
        Err(err) => err,
    };
    let err = match err {
        ClientError::Http(err) => err,
        _ => return Response::error(format!("{:?}", err), StatusCode::BAD_REQUEST.as_u16()),
    };
     match err.as_ref() {
        rspotify::http::HttpError::StatusCode(response) => Response::error(format!("{:?}", err), response.status().as_u16()),
        rspotify::http::HttpError::Client(_) => Response::error(format!("{:?}", err), StatusCode::BAD_REQUEST.as_u16()),
    }
}
