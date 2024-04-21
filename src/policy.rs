use reqwest::Url;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer};

/// Redirect policy for `UrlTrackCleaner`
#[derive(Clone, Debug, PartialEq)]
pub enum RedirectPolicy {
    None,
    All,
    Domains(Vec<String>),
}

impl Default for RedirectPolicy {
    fn default() -> Self {
        RedirectPolicy::None
    }
}

impl RedirectPolicy {
    pub fn test_url(&self, url: &Url) -> bool {
        match self {
            RedirectPolicy::None => false,
            RedirectPolicy::Domains(domains) => {
                let domain = url.domain().unwrap_or("");
                domains.iter().any(|d| domain.ends_with(d))
            }
            _ => true,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for RedirectPolicy {
    fn deserialize<D>(deserializer: D) -> Result<RedirectPolicy, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawTypes<'a> {
            None,
            EnumStr(&'a str),
            StringVec(Vec<String>),
        }

        Ok(match RawTypes::deserialize(deserializer)? {
            RawTypes::None => RedirectPolicy::None,
            RawTypes::EnumStr(s) => match s {
                "none" => RedirectPolicy::None,
                "*" => RedirectPolicy::All,
                _ => {
                    return Err(serde::de::Error::custom(format!(
                        "unknown redirect policy: {}",
                        s
                    )))
                },
            },
            RawTypes::StringVec(v) => RedirectPolicy::Domains(v),
        })
    }
}

#[cfg(feature = "serde")]
impl Serialize for RedirectPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RedirectPolicy::None => serializer.serialize_str("none"),
            RedirectPolicy::All => serializer.serialize_str("*"),
            RedirectPolicy::Domains(v) => v.serialize(serializer),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::RedirectPolicy;

    #[tokio::test]
    pub async fn parse_redirect_policy() -> Result<(), anyhow::Error> {
        let a = serde_json::to_string(&RedirectPolicy::All)?;
        let b = serde_json::to_string(&RedirectPolicy::Domains(vec![
            "b23.tv".into(),
            "t.cn".into(),
        ]))?;

        println!("a: {}", a);
        println!("b: {}", b);

        let a = serde_json::from_str::<RedirectPolicy>(&a)?;
        let b = serde_json::from_str::<RedirectPolicy>(&b)?;

        println!("a: {:?}", a);
        println!("b: {:?}", b);

        assert_eq!(a, RedirectPolicy::All);
        assert_eq!(
            b,
            RedirectPolicy::Domains(vec!["b23.tv".into(), "t.cn".into()])
        );

        Ok(())
    }
}
