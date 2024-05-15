use std::io::Cursor;

use axum::Json;
use image::ImageFormat;
use reqwest::{Client, Error};
use serde_json::{json, Value};
use sha256::digest;
use tokio::join;

use crate::util::RgbBuffer;

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
    cloudflare_url: &str,
    image: RgbBuffer,
) -> Result<String, Error> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .expect("image should be valid");

    let hash = digest(&bytes);

    let request = client
        .post(format!("{}/api/images/?key={}", cloudflare_url, hash))
        .body(bytes)
        .send();

    let _ = request.await?;

    Ok(hash)
}
