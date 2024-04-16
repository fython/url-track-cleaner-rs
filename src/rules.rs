use anyhow::Result;
use serde::{Deserialize, Serialize, Serializer};

/// A rule defines how to reserve queries in urls matching the rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveRule {
    #[serde(deserialize_with = "deserialize_regex", serialize_with = "serialize_regex")]
    pub url_match: regex::Regex,
    pub reserve_queries: Vec<String>,
}

impl ReserveRule {
    pub fn new(url_match: regex::Regex, reserve_queries: Vec<String>) -> Self {
        Self {
            url_match,
            reserve_queries,
        }
    }

    pub fn new_with_regex(url_match: &str, reserve_queries: Vec<String>) -> Result<Self> {
        let url_match = regex::Regex::new(url_match)?;
        Ok(Self {
            url_match,
            reserve_queries,
        })
    }
}

fn serialize_regex<S>(v: &regex::Regex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(v.as_str())
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<regex::Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    regex::Regex::new(&s).map_err(serde::de::Error::custom)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_rule_deserialize() -> Result<()> {
        let rule_str = r#"{"url_match":"^http(s)?://www.bilibili.com/.*","reserve_queries":["t"]}"#;
        let rule: ReserveRule = serde_json::from_str(rule_str)?;
        println!("{:?}", rule);
        assert_eq!(rule.reserve_queries, vec!["t"]);
        assert!(rule.url_match.is_match("https://www.bilibili.com/video/BV11111?t=360&track_id=2"));
        assert!(!rule.url_match.is_match("https://www.acfun.tv/video/BV11111"));
        Ok(())
    }
}
