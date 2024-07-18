use std::io::Read;
use actix_web::{get, HttpResponse, Responder, web, dev, http};
use serde::Deserialize;

#[derive(Deserialize)]
struct Query {
    url: String
}

#[get("/api/image")]
pub async fn proxy_image(query: web::Query<Query>, info: dev::ConnectionInfo) -> impl Responder {
    if query.url.starts_with(format!("{}://{}/api/image", info.scheme(), info.host()).as_str()) {
        return HttpResponse::BadRequest().body("Loop detected")
    }

    match ureq::get(&query.url).call() {
        Ok(response) => {
            if !response.content_type().starts_with("image/") {
                return HttpResponse::BadRequest().body("Passed url has no image content type");
            }
            let content_type = response.content_type().to_owned();
            let mut buffer = vec![];
            match response.into_reader().read_to_end(&mut buffer)  {
                Ok(_) =>
                    HttpResponse::Ok()
                        .append_header((http::header::CONTENT_TYPE, content_type))
                        .body(buffer),
                Err(error) => HttpResponse::BadRequest().body(format!("Could not proxy image: {}", error))
            }
        },
        Err(error) => HttpResponse::BadRequest().body(format!("Could not proxy image: {}", error))
    }
}