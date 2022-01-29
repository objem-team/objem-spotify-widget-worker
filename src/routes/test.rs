use http::StatusCode;
use rspotify::{
    clients::OAuthClient,
    model::{AdditionalType},
};
use worker::{Request, Response, Result, RouteContext};

use crate::utils;

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let spotify = match utils::get_auth_code_spotify(&req, &ctx).await {
        Ok(spotify) => spotify,
        Err(error_response) => return error_response,
    };
    let result = match spotify
        .current_playback(
            None,
            Some(&vec![AdditionalType::Track, AdditionalType::Episode]),
        )
        .await
    {
        Ok(context) => context,
        Err(err) => {
            return Response::error(
                format!("UNAUTHORIZED \n {:?}", err),
                StatusCode::UNAUTHORIZED.as_u16(),
            )
        }
    };

    match result {
        Some(result) => Response::from_json(&result),
        None => Response::error("No Playing context", StatusCode::BAD_REQUEST.as_u16()),
    }
}
