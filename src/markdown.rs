#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    Sentence {
        sentence_id: String,
    },
    Word {
        sentence_id: String,
        word_id: String,
    },
}

pub fn parse_target(uri: &str) -> Option<LinkTarget> {
    if let Some(id) = uri.strip_prefix("bibi://sentence/") {
        if !id.is_empty() && !id.contains('/') {
            return Some(LinkTarget::Sentence {
                sentence_id: id.into(),
            });
        }
    }
    if let Some(ids) = uri.strip_prefix("bibi://word/") {
        let (sentence_id, word_id) = ids.split_once('/')?;
        if !sentence_id.is_empty() && !word_id.is_empty() && !word_id.contains('/') {
            return Some(LinkTarget::Word {
                sentence_id: sentence_id.into(),
                word_id: word_id.into(),
            });
        }
    }
    None
}

pub fn target_uri(target: &LinkTarget) -> String {
    match target {
        LinkTarget::Sentence { sentence_id } => format!("bibi://sentence/{sentence_id}"),
        LinkTarget::Word {
            sentence_id,
            word_id,
        } => format!("bibi://word/{sentence_id}/{word_id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sentence_and_word_targets() {
        assert_eq!(
            parse_target("bibi://sentence/s01"),
            Some(LinkTarget::Sentence {
                sentence_id: "s01".into()
            })
        );
        assert_eq!(
            parse_target("bibi://word/s01/w02"),
            Some(LinkTarget::Word {
                sentence_id: "s01".into(),
                word_id: "w02".into()
            })
        );
    }

    #[test]
    fn rejects_external_links() {
        assert!(parse_target("https://example.com").is_none());
    }
}
