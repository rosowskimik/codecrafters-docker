use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_compression::tokio::bufread::GzipDecoder;
use reqwest::{header::ACCEPT, Client};
use serde::{Deserialize, Serialize};

use anyhow::{bail, Result};
use futures::TryStreamExt;
use tokio::{io::BufReader, task::JoinHandle};
use tokio_tar::Archive;
use tokio_util::compat::FuturesAsyncReadCompatExt;

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    token: String,
    access_token: Option<String>,
    expires_in: Option<u64>,
    issued_at: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    media_type: String,
    size: u64,
    digest: String,
    #[serde(default)]
    urls: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestResponse {
    schema_version: u8,
    media_type: String,
    config: Item,
    layers: Vec<Item>,
}

#[derive(Debug)]
struct Shared {
    client: Client,
    base_url: String,
    target_dir: PathBuf,
    token: String,
}

impl Shared {
    fn arc_new(
        client: Client,
        base_url: impl ToString,
        target_dir: impl Into<PathBuf>,
        token: impl ToString,
    ) -> Arc<Self> {
        Arc::new(Self {
            client,
            base_url: base_url.to_string(),
            target_dir: target_dir.into(),
            token: token.to_string(),
        })
    }
}

pub async fn fetch_image(container: &str, target_dir: &Path) -> Result<()> {
    // println!("Starting fetch image");
    let client = Client::new();

    let (image, tag) = match container.split_once(':') {
        Some(v) => v,
        None => (container, "latest"),
    };

    let (auth, manifest) = fetch_manifest(&client, image, tag).await?;

    let base_url = format!(
        "https://registry.hub.docker.com/v2/library/{}/blobs/",
        image
    );

    let shared = Shared::arc_new(client, base_url, target_dir, auth.token);

    let handles: Vec<JoinHandle<Result<()>>> = manifest
        .layers
        .into_iter()
        .map(|layer| {
            let shared = shared.clone();
            tokio::spawn(async move {
                let resp = shared
                    .client
                    .get(&format!("{}{}", shared.base_url, layer.digest))
                    .bearer_auth(&shared.token)
                    .send()
                    .await?
                    .bytes_stream()
                    .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
                    .into_async_read()
                    .compat();

                let reader = BufReader::new(resp);
                let reader = GzipDecoder::new(reader);
                let mut reader = Archive::new(reader);

                reader.unpack(&shared.target_dir).await?;
                Ok(())
            })
        })
        .collect();

    for handle in handles {
        handle.await??;
    }

    Ok(())
}

async fn fetch_auth(client: &Client, image: &str) -> Result<AuthResponse> {
    // println!("Starting fetch auth");
    // Www-Authenticate: Bearer realm="https://auth.docker.io/token",service="registry.docker.io",scope="repository:samalba/my-app:pull,push"
    // let auth = header.to_str()?;

    // let (_, auth) = auth.split_at(7);

    // // realm="https://auth.docker.io/token"
    // let (_, auth) = auth.split_once('"').context("")?;
    // let (realm, auth) = auth.split_once('"').context("")?;

    // // ,service="registry.docker.io"
    // let (_, auth) = auth.split_once('"').context("")?;
    // let (service, auth) = auth.split_once('"').context("")?;

    // // ,scope="repository:samalba/my-app:pull,push"
    // let (_, auth) = auth.split_once('"').context("")?;
    // let (scope, _) = auth.split_once('"').context("")?;

    // println!("Ending fetch auth");
    let auth_url = format!(
        "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{}:pull",
        image
    );
    Ok(client
        .get(auth_url)
        .send()
        .await?
        .json::<AuthResponse>()
        .await?)
}

async fn fetch_manifest(
    client: &Client,
    image: &str,
    tag: &str,
) -> Result<(AuthResponse, ManifestResponse)> {
    // println!("Starting fetch manifest");
    // let resp = client.get(&url).send().await?;
    // assert!(resp.status().as_u16() == 401);
    // dbg!(&resp);

    // let auth_header = resp
    //     .headers()
    //     .get(WWW_AUTHENTICATE)
    //     .context("No Www-Authenticate header")?;
    let auth = fetch_auth(client, image).await?;

    let manifest_url = format!(
        "https://registry.hub.docker.com/v2/library/{}/manifests/{}",
        image, tag
    );
    let resp = client
        .get(&manifest_url)
        .bearer_auth(&auth.token)
        .header(
            ACCEPT,
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .send()
        .await?;

    if resp.status().as_u16() == 404 {
        bail!("Not found!");
    }

    // println!("Ending fetch mainfest");
    Ok((auth, resp.json().await?))
}
