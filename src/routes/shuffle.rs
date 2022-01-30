use http::StatusCode;
use rspotify::{clients::OAuthClient, ClientError};
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

use crate::utils;

pub async fn handler(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let form_data = match req.form_data().await{
        Ok(data) => data,
        Err(err) => return Response::error(format!("{:?}",err), StatusCode::BAD_REQUEST.as_u16()),
    };

    let form_entory = match form_data.get("shaffleState"){
        Some(mode) => mode,
        None => return Response::error("BadRequest", StatusCode::BAD_REQUEST.as_u16()),
    };

    let shaffle_state = match form_entory {
        worker::FormEntry::Field(mode) =>mode,
        worker::FormEntry::File(_) => return Response::error("BadRequest", StatusCode::BAD_REQUEST.as_u16()),
    };
    let shaffle_state :bool = match shaffle_state.parse(){
        Ok(mode) => mode,
        Err(err)=> return Response::error(format!("{:?}",err), StatusCode::BAD_REQUEST.as_u16()),
    };
    let spotify = match utils::get_auth_code_spotify(&req, &ctx).await {
        Ok(spotify) => spotify,
        Err(error_response) => return error_response,
    };
    let err = match spotify.shuffle(shaffle_state,None).await {
        Ok(_) => return Response::from_json(&json!(shaffle_state)),
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


