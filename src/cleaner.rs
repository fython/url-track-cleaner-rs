use std::str::FromStr;

use anyhow::{format_err, Result};
use reqwest::{header, redirect, IntoUrl, Url};
use serde::{Deserialize, Serialize};
use tokio::net::lookup_host;

use crate::policy::RedirectPolicy;
use crate::rules::ReserveRule;

const DEFAULT_USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36";

/// Cleaner for tracking url
///
/// # Builder
///
/// This struct can be constructed by `UrlTrackCleanerBuilder`.
///
/// # Serialization
///
/// You can deserialize builder by serde. Then build this struct from the builder.
///
/// # Example
///
/// ```
/// #use url_track_cleaner::{UrlTrackCleaner, UrlTrackCleanerBuilder, ReserveRule, RedirectPolicy};
///
/// ##[tokio::main]
/// #async fn main() {
/// let reserve_rules: Vec<ReserveRule> = vec![ReserveRule::new_with_regex(
///   r#"^http(s)?://www.bilibili.com/.*"#,
///   vec!["t".to_string()],
/// ).expect("failed to create reserve rule")];
/// let cleaner = UrlTrackCleaner::builder()
///   .reserve_rules(reserve_rules)
///   .build();
/// let cleaned = cleaner
///   .do_clean("https://www.bilibili.com/video/BV11111?t=360&track_id=2")
///   .await
///   .expect("failed to clean url");
/// println!("cleaned url: {}", cleaned);
/// #}
/// ```
#[derive(Clone, Debug)]
pub struct UrlTrackCleaner {
    follow_redirect: RedirectPolicy,
    reserve_rules: Vec<ReserveRule>,
    user_agent: String,
    client: reqwest::Client,
}

impl Default for UrlTrackCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlTrackCleaner {
    /// Construct a new `UrlTrackCleaner`
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .redirect(redirect::Policy::none())
            .build()
            .unwrap();
        Self {
            follow_redirect: Default::default(),
            reserve_rules: Default::default(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            client,
        }
    }

    /// Construct a builder for `UrlTrackCleaner`
    ///
    /// This is same as `UrlTrackCleanerBuilder::new()`
    pub fn builder() -> UrlTrackCleanerBuilder {
        UrlTrackCleanerBuilder::new()
    }

    /// Clean the url by the reserve rules.
    pub async fn do_clean<U>(&self, url: U) -> Result<Url>
    where
        U: IntoUrl,
    {
        let mut url = url.into_url()?;
        // test if the redirection check should be skipped
        if !self.skip_redirect(&url).await {
            let rsp = self
                .client
                .get(url)
                .header(header::USER_AGENT, &self.user_agent)
                .send()
                .await?;
            // Check if the response is a redirection. If it is, get the location header and parse it as the final url.
            url = if rsp.status().is_redirection() {
                let location = rsp
                    .headers()
                    .get(header::LOCATION)
                    .ok_or_else(|| format_err!("no location found"))?;
                Url::from_str(location.to_str()?)?
            } else {
                rsp.url().to_owned()
            };
        }
        Ok(self.do_clean_without_http_check(url))
    }

    async fn skip_redirect(&self, url: &Url) -> bool {
        if !self.follow_redirect.test_url(url) {
            return true;
        }
        if let Some(host) = url.host_str() {
            if let Ok(host) = lookup_host(host).await {
                return host.count() < 1;
            }
        }
        return true;
    }

    /// Clean the url by the reserve rules without http check.
    fn do_clean_without_http_check(&self, url: Url) -> Url {
        // Check if the url matches any reserve rules
        for rule in &self.reserve_rules {
            if rule.url_match.is_match(&url.to_string()) {
                let mut url = url;
                let mut query = url.query_pairs().collect::<Vec<_>>();
                query.retain(|(k, _)| rule.reserve_queries.contains(&k.to_string()));
                url.set_query(Some(
                    &query
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&"),
                ));
                return url;
            }
        }
        // If the url does not match any reserve rules, remove all queries
        let mut url = url;
        url.set_query(None);
        url
    }
}

/// Builder for `UrlTrackCleaner`
///
/// # Serialization
///
/// This struct can be serialized and deserialized by serde.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UrlTrackCleanerBuilder {
    follow_redirect: RedirectPolicy,
    reserve_rules: Vec<ReserveRule>,
    user_agent: Option<String>,
}

impl Default for UrlTrackCleanerBuilder {
    fn default() -> Self {
        Self {
            follow_redirect: Default::default(),
            reserve_rules: Default::default(),
            user_agent: None,
        }
    }
}

impl UrlTrackCleanerBuilder {
    /// Construct a new `UrlTrackCleanerBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the redirect policy for the cleaner
    pub fn follow_redirect(mut self, follow_redirect: RedirectPolicy) -> Self {
        self.follow_redirect = follow_redirect;
        self
    }

    /// Set the reserve rules for the cleaner
    pub fn reserve_rules(mut self, reserve_rules: Vec<ReserveRule>) -> Self {
        self.reserve_rules = reserve_rules;
        self
    }

    /// Set the user agent for the cleaner
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Build the `UrlTrackCleaner`
    pub fn build(self) -> UrlTrackCleaner {
        let mut cleaner = UrlTrackCleaner::default();
        cleaner.follow_redirect = self.follow_redirect;
        cleaner.reserve_rules = self.reserve_rules;
        if let Some(user_agent) = self.user_agent {
            cleaner.user_agent = user_agent;
        }
        cleaner
    }
}
