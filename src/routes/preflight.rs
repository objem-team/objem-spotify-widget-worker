use worker::{Request, Response, Result, RouteContext};

pub fn handler(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::empty()
}