use std::io::Cursor;

use fastly::error::{anyhow, bail};
use fastly::http::{Method, StatusCode};
use fastly::{mime, Error, Request, Response};
use rustagram::image;
use rustagram::image::io::Reader as ImageReader;
use rustagram::{FilterType, RustagramFilter};

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    // Pattern match on the path...
    match (req.get_method(), req.get_path()) {
        // If request is to the `/` path...
        (&Method::GET, "/") => Ok(Response::from_status(StatusCode::OK)
            .with_content_type(mime::TEXT_HTML_UTF_8)
            .with_body(include_str!("index.html"))),
        (&Method::GET, "/app.js") => Ok(Response::from_status(StatusCode::OK)
            .with_content_type(mime::APPLICATION_JAVASCRIPT)
            .with_body(include_str!("app.js"))),

        (&Method::POST, "/image") => convert_image(req),

        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}

pub fn convert_image(mut req: Request) -> Result<Response, Error> {
    let filter: FilterType = req
        .get_query_parameter("filter")
        .ok_or_else(|| anyhow!("missing filter"))?
        .parse()
        .map_err(|_| anyhow!("invalid filter"))?;

    if !req.has_body() {
        bail!("missing image");
    }

    let body = req.take_body();
    let body = body.into_bytes();

    let img = ImageReader::new(Cursor::new(body))
        .with_guessed_format()
        .map_err(|_| anyhow!("not an image"))?;

    let img = img.decode().map_err(|_| anyhow!("not an image"))?;

    let img = img.thumbnail(500, 500);
    let out = img.to_rgba8().apply_filter(filter);
    let mut bytes: Vec<u8> = Vec::new();
    out.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;

    Ok(Response::from_status(StatusCode::OK)
        .with_body(bytes)
        .with_content_type(mime::IMAGE_PNG))
}
