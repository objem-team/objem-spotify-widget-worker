use http::StatusCode;
use rspotify::clients::OAuthClient;
use worker::{Request,Response, RouteContext,Result};

use crate::utils;

pub async fn handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let spotify = match utils::get_auth_code_spotify(&req, &ctx).await{
         Ok(spotify) => spotify,
         Err(error_response) => return error_response,
     };
     let mut response =match spotify.next_track(None).await {
        Ok(_) => Response::ok(true.to_string()).unwrap(),
        Err(err) =>return Response::error(format!("{:?}",err), StatusCode::BAD_REQUEST.as_u16()),
     };
     utils::append_cors_header(response.headers_mut())?;
     Ok(response)
}