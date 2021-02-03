use std::collections::HashSet;
use std::str::FromStr;

use log::info;

use crate::collections::LineRange;

#[derive(Debug)]
pub struct KeywordMatcher {
    contains_at: bool,
    character_set: HashSet<char>,
    // TODO: Ignored characters.
}

#[allow(dead_code)]
impl KeywordMatcher {
    pub fn match_char(&self, c: &char) -> bool {
        // return any(x == c for x in self.character_set)
        self.character_set.contains(&c)
            || (self.contains_at && (u32::from(*c) > 255 || c.is_alphabetic()))
    }

    pub fn find(&self, line: &str, cursor: u64) -> LineRange {
        let cursor = cursor as usize;

        let mut start = cursor;
        let mut finish = cursor;

        // TODO(tjdevries): Need to handle multibyte problems here...
        let char_vec: Vec<char> = line.chars().collect();
        for index in (0..cursor as usize).rev() {
            if !self.match_char(&char_vec[index]) {
                start = index + 1;
                break;
            }
        }

        for index in cursor + 1..line.len() {
            finish = index;
            if !self.match_char(&char_vec[index]) {
                break;
            }
        }

        return LineRange { start, finish };
    }
}

fn convert_numeric_string(numeric_str: &str) -> char {
    std::char::from_u32(
        numeric_str
            .parse::<u32>()
            .expect(&format!("Already numeric {}", numeric_str)),
    )
    .expect("Valid char")
}

#[derive(Debug)]
pub enum KeywordError {
    ContainsTripleCommasPlzNo,
}

impl FromStr for KeywordMatcher {
    type Err = KeywordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(",,,") {
            return Err(KeywordError::ContainsTripleCommasPlzNo);
        }

        let mut contains_at = false;

        let mut character_set = HashSet::new();
        for section in s.split(',') {
            match section {
                "@" => {
                    contains_at = true;
                }
                "@-@" => {
                    character_set.insert('@');
                }
                section if section.contains("-") => {
                    // TODO: OK, so we can split the item on "-"
                    // Then do the range of the things here.
                    //  alpha ranges might be hard?...
                    //  Honestly, we could leave these for later.
                    //  numeric ranges should be pretty easy.
                    let subsections: Vec<&str> = section.split("-").collect();
                    info!("Current subsections: {:?} {:?}", section, subsections);

                    let start = subsections[0].parse::<u32>().expect("Already numeric");
                    let finish = subsections[1].parse::<u32>().expect("Already numeric");

                    info!("    Start {} / Finish {}", start, finish);

                    for i in start..finish + 1 {
                        let new_char = std::char::from_u32(i).expect("This has to be valid");
                        info!("Found new_char: {}", new_char);
                        character_set.insert(new_char);
                    }
                }
                section if section.len() > 0 && section.chars().all(|x| x.is_numeric()) => {
                    character_set.insert(convert_numeric_string(section));
                }
                section if section.len() > 0 && section.chars().all(|x| x.is_alphabetic()) => {
                    section.chars().for_each(|x| {
                        character_set.insert(x);
                    });
                }
                _ => {
                    // println!("What is this situation...");
                }
            }
        }

        Ok(KeywordMatcher {
            contains_at,
            character_set,
        })
    }
}

pub fn transform(iskeyword: &str) -> KeywordMatcher {
    iskeyword.parse().expect("this always works")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_iskeyword() {
        let matcher = transform("");
        assert!(!matcher.match_char(&'a'));
    }

    #[test]
    fn test_matches_single_char() {
        let matcher = transform("a");
        assert!(matcher.match_char(&'a'));
    }

    #[test]
    fn test_matches_single_char_in_list() {
        let matcher = transform("a,b,c");
        assert!(matcher.match_char(&'a'));
    }

    #[test]
    fn test_matches_single_char_in_list_as_number() {
        let matcher = transform("97,b,c");
        assert!(matcher.match_char(&'a'));
        assert!(matcher.match_char(&'b'));
        assert!(matcher.match_char(&'c'));

        assert!(!matcher.match_char(&'d'));
    }

    #[test]
    fn test_matches_single_char_in_list_with_range() {
        let matcher = transform("90-100");
        assert!(matcher.match_char(&'a'));
        assert!(matcher.match_char(&'b'));
        assert!(matcher.match_char(&'c'));

        assert!(!matcher.match_char(&'A'));
    }

    #[test]
    fn test_find_words() {
        let matcher = transform("65-81,91-116");

        assert!(matcher.match_char(&'a'));
        assert!(matcher.match_char(&'A'));

        assert_eq!(
            matcher.find("hello world", 7),
            LineRange {
                start: 7,
                finish: 10
            }
        );
    }
}
