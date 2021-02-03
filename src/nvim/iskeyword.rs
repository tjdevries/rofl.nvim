use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug)]
pub struct KeywordMatcher {
    contains_at: bool,
    character_set: HashSet<char>,
}

#[allow(dead_code)]
impl KeywordMatcher {
    pub fn match_char(&self, c: &char) -> bool {
        // return any(x == c for x in self.character_set)
        self.character_set.contains(&c) || (self.contains_at && u32::from(*c) > 255)
    }

    pub fn find<'a>(&self, line: &'a str, _cursor: u64) -> &'a str {
        return line;
    }
}

fn convert_numeric_string(numeric_str: &str) -> char {
    std::char::from_u32(numeric_str.parse::<u32>().expect("Already numeric")).expect("Valid char")
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
                    println!("What is this situation...");
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
}
