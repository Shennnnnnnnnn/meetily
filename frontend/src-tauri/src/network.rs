use std::sync::{OnceLock, RwLock};

use anyhow::{anyhow, Result};
use reqwest::ClientBuilder;
use url::Url;

static DOWNLOAD_PROXY: OnceLock<RwLock<Option<String>>> = OnceLock::new();

fn proxy_cell() -> &'static RwLock<Option<String>> {
    DOWNLOAD_PROXY.get_or_init(|| RwLock::new(None))
}

pub fn validate_download_proxy(proxy: Option<&str>) -> Result<Option<String>> {
    let Some(proxy) = proxy.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let parsed = Url::parse(proxy).map_err(|error| anyhow!("Invalid proxy address: {}", error))?;
    match parsed.scheme() {
        "http" | "https" => Ok(Some(proxy.trim_end_matches('/').to_string())),
        scheme => Err(anyhow!(
            "Unsupported proxy scheme '{}'; use an HTTP or HTTPS proxy",
            scheme
        )),
    }
}

pub fn set_download_proxy(proxy: Option<&str>) -> Result<Option<String>> {
    let normalized = validate_download_proxy(proxy)?;
    let mut guard = proxy_cell()
        .write()
        .map_err(|_| anyhow!("Download proxy lock is poisoned"))?;
    *guard = normalized.clone();
    Ok(normalized)
}

pub fn get_download_proxy() -> Option<String> {
    proxy_cell().read().ok().and_then(|guard| guard.clone())
}

pub fn configure_download_client(builder: ClientBuilder) -> Result<ClientBuilder> {
    match get_download_proxy() {
        Some(proxy) => Ok(builder.proxy(reqwest::Proxy::all(&proxy)?)),
        None => Ok(builder),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_http_and_https_proxy_addresses() {
        assert_eq!(
            validate_download_proxy(Some("http://127.0.0.1:7890/"))
                .unwrap()
                .as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert!(validate_download_proxy(Some("socks5://127.0.0.1:1080")).is_err());
        assert!(validate_download_proxy(Some("not a url")).is_err());
        assert_eq!(validate_download_proxy(Some("  ")).unwrap(), None);
    }
}
