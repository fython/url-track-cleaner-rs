mod cleaner;
mod rules;

pub use cleaner::{RedirectPolicy, UrlTrackCleaner, UrlTrackCleanerBuilder};
pub use rules::ReserveRule;

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    pub async fn test_clean_b23() -> Result<()> {
        let cleaner = UrlTrackCleaner::default();
        let cleaned = cleaner.do_clean("https://b23.tv/iFW9atP").await?;
        println!("cleaned url: {}", cleaned);
        Ok(())
    }

    #[tokio::test]
    pub async fn test_clean_bilibili_with_reserving_timestamp() {
        let reserve_rules: Vec<ReserveRule> = vec![
            ReserveRule::new_with_regex(
                r#"^http(s)?://www.bilibili.com/.*"#,
                vec!["t".to_string()],
            ).expect("failed to create reserve rule"),
        ];
        let cleaner = UrlTrackCleaner::builder()
            .reserve_rules(reserve_rules)
            .build();
        let cleaned = cleaner.do_clean("https://www.bilibili.com/video/BV11111?t=360&track_id=2").await
            .expect("failed to clean url");
        println!("cleaned url: {}", cleaned);
        assert_eq!(cleaned.query_pairs().count(), 1);
        assert_eq!(cleaned.query_pairs().next().unwrap().0, "t");
        assert_eq!(format!("{}{}", cleaned.domain().unwrap(), cleaned.path()), "www.bilibili.com/video/BV11111");
    }
}
