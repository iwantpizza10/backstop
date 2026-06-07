use std::error::Error;

use reqwest::Url;
use semver::{Version, VersionReq};
use serde::Deserialize;

const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

#[derive(Deserialize)]
struct GithubResponse {
    name: String,
}

async fn check_for_updates_internal() -> Result<bool, Box<dyn Error>> {
    let url = Url::parse("https://api.github.com/repos/iwantpizza10/backstop/releases/latest")?;
    let new_req = VersionReq::parse(concat!(">", env!("CARGO_PKG_VERSION")))?;

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    let text = client
        .get(url)
        .send()
        .await?
        .text()
        .await?;

    if let Ok(x) = serde_json::from_str::<GithubResponse>(&text) {
        let name_without_v = x.name.replace("v", "");
        let ver = Version::parse(&name_without_v)?;

        Ok(new_req.matches(&ver))
    } else {
        Ok(false)
    }
}

pub async fn check_for_updates() -> bool {
    check_for_updates_internal().await.unwrap_or_default()
}
