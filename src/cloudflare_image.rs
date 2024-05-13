use std::io::Cursor;

use axum::Json;
use image::ImageFormat;
use reqwest::{multipart::Form, Client, Error};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::join;

use crate::util::RgbBuffer;

#[derive(Deserialize)]
struct CloudflareImageResult {
    variants: Vec<String>,
}

#[derive(Deserialize)]
struct CloudflareImageResponse {
    result: CloudflareImageResult,
}

pub struct SolutionPair {
    unsolved: RgbBuffer,
    solved: RgbBuffer,
}

impl SolutionPair {
    pub fn new(unsolved: RgbBuffer, solved: RgbBuffer) -> Self {
        SolutionPair { unsolved, solved }
    }
}

pub async fn serve_pair(
    client: &Client,
    cloudflare_id: &str,
    SolutionPair { solved, unsolved }: SolutionPair,
) -> Result<Json<Value>, Error> {
    let (unsolved_response, solved_response) = join!(
        serve_image(client, cloudflare_id, unsolved),
        serve_image(client, cloudflare_id, solved)
    );

    Ok(Json(
        json!({ "unsolved": unsolved_response?, "solved": solved_response? }),
    ))
}

pub async fn serve_image(
    client: &Client,
    cloudflare_id: &str,
    image: RgbBuffer,
) -> Result<Vec<String>, Error> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .expect("image should be valid");
    let form = Form::new().part("file", reqwest::multipart::Part::bytes(bytes));

    let request = client
        .post(format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/images/v1",
            cloudflare_id
        ))
        .multipart(form)
        .send();

    let unsolved_response = request.await?.json::<CloudflareImageResponse>().await?;

    Ok(unsolved_response.result.variants)
}
