use std::io::Cursor;

use image::{ImageBuffer, ImageFormat, Rgb};
use reqwest::{multipart::Form, Client, Error};
use serde::Deserialize;

#[derive(Deserialize)]
struct CloudflareImageResult {
    variants: Vec<String>,
}

#[derive(Deserialize)]
struct CloudflareImageResponse {
    result: CloudflareImageResult,
}

pub async fn serve_image(
    client: &Client,
    cloudflare_id: &str,
    image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> Result<Vec<String>, Error> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Jpeg)
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
