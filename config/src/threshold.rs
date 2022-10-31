use anyhow::Context;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MissedBlockThreshold {
    pub numerator: u64,
    pub denominator: u64,
}

impl TryFrom<String> for MissedBlockThreshold {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains("/") {
            let mut nums = value.split("/");
            let numerator = nums
                .next()
                .unwrap()
                .parse()
                .with_context(|| format!("illegal missed_block_threshold format: {}", value))?;
            let denominator = nums
                .next()
                .unwrap()
                .parse()
                .with_context(|| format!("illegal missed_block_threshold format: {}", value))?;
            Ok(Self {
                numerator,
                denominator,
            })
        } else {
            let v = value
                .as_str()
                .parse()
                .with_context(|| format!("illegal missed_block_threshold format: {}", value))?;
            Ok(Self {
                numerator: v,
                denominator: v,
            })
        }
    }
}

impl Default for MissedBlockThreshold {
    /// alert any missed block
    fn default() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_threshold_single() {
        let threshold = MissedBlockThreshold::default();
        assert_eq!(
            threshold,
            MissedBlockThreshold {
                numerator: 1,
                denominator: 1,
            }
        );
        let data = vec![
            (1, true),
            (2, true),
            (3, false),
            (4, false),
            (5, true),
            (6, false),
        ];
        let expected = vec![
            (1, false),
            (2, false),
            (3, true),
            (4, true),
            (5, false),
            (6, true),
        ];
        let mut result = vec![];
        let mut missed_block_heights = vec![];
        for (block_height, signed) in data {
            let lowest = block_height - (threshold.denominator as i64) + 1;
            missed_block_heights = missed_block_heights
                .clone()
                .into_iter()
                .filter(|&h| h >= lowest)
                .collect();
            if signed {
                result.push((block_height, false));
            } else {
                missed_block_heights.push(block_height);
                result.push((
                    block_height,
                    missed_block_heights.len() >= threshold.numerator as usize,
                ));
            }
        }
        assert_eq!(result, expected);
    }

    #[test]
    fn test_threshold_2() {
        let threshold = "2".to_owned();
        let threshold = MissedBlockThreshold::try_from(threshold).unwrap();
        assert_eq!(
            threshold,
            MissedBlockThreshold {
                numerator: 2,
                denominator: 2,
            }
        );
        let data = vec![
            (1, true),
            (2, true),
            (3, false),
            (4, false),
            (5, true),
            (6, false),
        ];
        let expected = vec![
            (1, false),
            (2, false),
            (3, false),
            (4, true),
            (5, false),
            (6, false),
        ];
        let mut result = vec![];
        let mut missed_block_heights = vec![];
        for (block_height, signed) in data {
            let lowest = block_height - (threshold.denominator as i64) + 1;
            missed_block_heights = missed_block_heights
                .clone()
                .into_iter()
                .filter(|&h| h >= lowest)
                .collect();
            if signed {
                result.push((block_height, false));
            } else {
                missed_block_heights.push(block_height);
                result.push((
                    block_height,
                    missed_block_heights.len() >= threshold.numerator as usize,
                ));
            }
        }
        assert_eq!(result, expected);
    }

    #[test]
    fn test_threshold_2_out_of_10() {
        let threshold = "2/10".to_owned();
        let threshold = MissedBlockThreshold::try_from(threshold).unwrap();
        assert_eq!(
            threshold,
            MissedBlockThreshold {
                numerator: 2,
                denominator: 10,
            }
        );
        let data = vec![
            (1, false),
            (2, true),
            (3, true),
            (4, true),
            (5, true),
            (6, true),
            (7, true),
            (8, true),
            (9, true),
            (10, false),
            (11, true),
            (12, true),
            (13, true),
            (14, true),
            (15, true),
            (16, true),
            (17, true),
            (18, true),
            (19, true),
            (20, true),
            (21, false),
        ];
        let expected = vec![
            (1, false),
            (2, false),
            (3, false),
            (4, false),
            (5, false),
            (6, false),
            (7, false),
            (8, false),
            (9, false),
            (10, true),
            (11, false),
            (12, false),
            (13, false),
            (14, false),
            (15, false),
            (16, false),
            (17, false),
            (18, false),
            (19, false),
            (20, false),
            (21, false),
        ];
        let mut result = vec![];
        let mut missed_block_heights = vec![];
        for (block_height, signed) in data {
            let lowest = block_height - (threshold.denominator as i64) + 1;
            missed_block_heights = missed_block_heights
                .clone()
                .into_iter()
                .filter(|&h| h >= lowest)
                .collect();
            if signed {
                result.push((block_height, false));
            } else {
                missed_block_heights.push(block_height);
                result.push((
                    block_height,
                    missed_block_heights.len() >= threshold.numerator as usize,
                ));
            }
        }
        assert_eq!(result, expected);
    }
}
