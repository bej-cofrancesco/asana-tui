//! Text processing utilities.
//!
//! This module contains utilities for processing and transforming text,
//! such as replacing profile URLs with user names.

use log::*;
use regex::Regex;
use std::collections::HashMap;

/// Replace profile URLs with "@ person name" in text.
///
/// URLs like "profiles/123456" or "https://app.asana.com/0/profile/123456"
/// become "@ person name" based on the provided user map.
///
/// # Arguments
/// * `text` - The text to process
/// * `user_map` - Map of user GID to user name
///
/// # Returns
/// The text with profile URLs replaced with "@ person name" format.
pub fn replace_profile_urls(text: &str, user_map: &HashMap<String, String>) -> String {
    // Pattern: https://app.asana.com/0/profile/{gid} or profiles/{gid}
    let profile_patterns = vec![
        r"https://app\.asana\.com/0/profile/(\d+)",
        r"profiles/(\d+)",
    ];

    let mut result = text.to_string();
    for pattern in profile_patterns {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to compile regex pattern '{}': {}", pattern, e);
                continue;
            }
        };
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                if let Some(gid_match) = caps.get(1) {
                    let gid = gid_match.as_str();
                    if let Some(name) = user_map.get(gid) {
                        format!("@{}", name)
                    } else {
                        // If user not found, keep the original URL
                        caps.get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default()
                    }
                } else {
                    caps.get(0)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default()
                }
            })
            .to_string();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_profile_urls_full_url() {
        let mut user_map = HashMap::new();
        user_map.insert("123456".to_string(), "John Doe".to_string());
        user_map.insert("789012".to_string(), "Jane Smith".to_string());

        let text = "Check with https://app.asana.com/0/profile/123456 about this task";
        let result = replace_profile_urls(text, &user_map);
        assert_eq!(result, "Check with @John Doe about this task");
    }

    #[test]
    fn test_replace_profile_urls_short_url() {
        let mut user_map = HashMap::new();
        user_map.insert("123456".to_string(), "John Doe".to_string());

        let text = "See profiles/123456 for details";
        let result = replace_profile_urls(text, &user_map);
        assert_eq!(result, "See @John Doe for details");
    }

    #[test]
    fn test_replace_profile_urls_multiple() {
        let mut user_map = HashMap::new();
        user_map.insert("123456".to_string(), "John Doe".to_string());
        user_map.insert("789012".to_string(), "Jane Smith".to_string());

        let text = "Contact https://app.asana.com/0/profile/123456 and profiles/789012";
        let result = replace_profile_urls(text, &user_map);
        assert_eq!(result, "Contact @John Doe and @Jane Smith");
    }

    #[test]
    fn test_replace_profile_urls_unknown_user() {
        let mut user_map = HashMap::new();
        user_map.insert("123456".to_string(), "John Doe".to_string());

        let text = "Check profiles/999999";
        let result = replace_profile_urls(text, &user_map);
        // Unknown user should keep original URL
        assert_eq!(result, "Check profiles/999999");
    }

    #[test]
    fn test_replace_profile_urls_no_matches() {
        let mut user_map = HashMap::new();
        user_map.insert("123456".to_string(), "John Doe".to_string());

        let text = "This is just regular text with no profile URLs";
        let result = replace_profile_urls(text, &user_map);
        assert_eq!(result, text);
    }

    #[test]
    fn test_replace_profile_urls_empty_map() {
        let user_map = HashMap::new();
        let text = "Check profiles/123456";
        let result = replace_profile_urls(text, &user_map);
        // Should keep original when user not found
        assert_eq!(result, text);
    }
}
