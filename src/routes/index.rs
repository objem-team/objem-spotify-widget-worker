use http::StatusCode;
use rspotify::{
    clients::OAuthClient,
    model::{AdditionalType, PlayableItem},
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

    let playing_context = match result {
        Some(result) => result,
        None => return Response::error("No Playing context", StatusCode::BAD_REQUEST.as_u16()),
    };

    let item = match playing_context.item {
        Some(item) => item,
        None => return Response::error("No Playing context", StatusCode::BAD_REQUEST.as_u16()),
    };

    let mut response = match item {
        PlayableItem::Track(track) => Response::from_json(&track).unwrap(),
        PlayableItem::Episode(episode) => Response::from_json(&episode).unwrap(),
    };

    response
        .headers_mut()
        //add CORS
        .append(
            "Access-Control-Allow-Origin",
            "https://tweetdeck.twitter.com",
        )?;
    response
        .headers_mut()
        .append("access-control-allow-credentials", "true")?;
    response
        .headers_mut()
        .append("access-control-allow-methods", "GET")?;
    Ok(response)
}
