#![expect(dead_code)]

use super::QualityScores;

enum QualityScoresRef<'a> {
    Raw(&'a [u8]),
    Offset(&'a [u8], u8),
    QualityScores(Box<dyn QualityScores + 'a>),
}

impl QualityScoresRef<'_> {
    fn is_empty(&self) -> bool {
        match self {
            Self::Raw(src) => src.is_empty(),
            Self::Offset(src, _) => src.is_empty(),
            Self::QualityScores(quality_scores) => quality_scores.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Raw(src) => src.len(),
            Self::Offset(src, _) => src.len(),
            Self::QualityScores(quality_scores) => quality_scores.len(),
        }
    }
}
