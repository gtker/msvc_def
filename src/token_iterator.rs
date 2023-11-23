use crate::parse_ref::RESERVED_WORDS;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct TokenIterator<'a> {
    pub rest: &'a str,
    pub offset: usize,
    pub version_token_encountered_tokens_ago: u8,
}

fn trim_start(s: &str) -> (usize, &str) {
    if let Some(i) = s.find(|a: char| !a.is_whitespace()) {
        (i, &s[i..])
    } else {
        (0, s)
    }
}

fn trim_end(s: &str) -> (usize, &str) {
    if let Some(i) = s.rfind(|a: char| !a.is_whitespace()) {
        (0, &s[..=i])
    } else {
        (s.len(), &s[..=s.len()])
    }
}

impl<'a> TokenIterator<'a> {
    pub fn new(s: &'a str) -> Self {
        let (offset, rest) = trim_start(s);
        Self {
            rest,
            offset,
            version_token_encountered_tokens_ago: 0,
        }
    }

    pub fn set_rest(&mut self, i: usize, peek: bool) {
        if !peek {
            let (offset, rest) = trim_start(&self.rest[i..]);
            self.rest = rest;
            self.offset += i + offset;
        }
    }

    pub fn remove_comment(&mut self) -> bool {
        if self.rest.starts_with(crate::parse_ref::COMMENT) {
            if let Some(i) = self.rest.find(|a| a == '\r') {
                if &self.rest[i..i + 1] == "\r\n" {
                    self.set_rest(i + 1, false);
                    return true;
                }
            }

            return if let Some(i) = self.rest.find(|a| a == '\n') {
                self.set_rest(i, false);
                true
            } else {
                false
            };
        }

        true
    }

    fn eat_token_inner(&mut self, peek: bool) -> Option<&'a str> {
        // Start of rest is not whitespace

        if self.rest.chars().all(|a| a.is_whitespace()) {
            return None;
        }

        if !self.remove_comment() {
            return None;
        }

        // rest is not whitespace, not a comment
        let find_matching_quote = self.rest.starts_with('"');

        if self.rest.starts_with(',')
            || self.rest.starts_with(':')
            || self.rest.starts_with('=')
            || (self.version_token_encountered_tokens_ago != 0 && self.rest.starts_with('.'))
        {
            let tmp = &self.rest[..1];
            self.set_rest(1, peek);

            if !self.remove_comment() {
                return None;
            }

            return Some(tmp);
        }

        if let Some(i) = self.rest[1..].find(|a: char| {
            (!find_matching_quote && a.is_whitespace())
                || (find_matching_quote && a == '"')
                || (!find_matching_quote
                    && (a == ','
                        || a == ':'
                        || a == '='
                        || (self.version_token_encountered_tokens_ago != 0 && a == '.')))
        }) {
            let offset = if find_matching_quote { 2 } else { 1 };

            // Deliberately leave in the starting quote in order to discern from real keywords
            let tmp = &self.rest[..i + 1];
            self.set_rest(i + offset, peek);

            if !self.remove_comment() {
                return None;
            }

            return Some(tmp);
        }

        // Don't need to update self.offset since empty strings should just return immediately
        let (_, tmp) = trim_end(self.rest);
        if !peek {
            self.rest = "";
        }

        Some(tmp)
    }

    fn eat_token_state_wrapper(&mut self, peek: bool) -> Option<&'a str> {
        let token = self.eat_token_inner(peek);
        if self.version_token_encountered_tokens_ago == 1 {
            self.version_token_encountered_tokens_ago = 2;
        } else if self.version_token_encountered_tokens_ago == 2 {
            self.version_token_encountered_tokens_ago = 0;
        }

        if let Some(token) = token {
            if token == "VERSION" {
                self.version_token_encountered_tokens_ago = 1;
            }
        }

        token
    }

    pub fn eat_token(&mut self) -> Option<&'a str> {
        self.eat_token_state_wrapper(false)
    }

    pub fn peek_token(&mut self) -> Option<&'a str> {
        self.eat_token_inner(true)
    }

    pub fn next_token_is(&mut self, token: &str) -> bool {
        let t = self.peek_token();
        match t {
            None => false,
            Some(s) => s == token,
        }
    }

    pub fn next_token_is_keyword(&mut self) -> bool {
        let t = self.peek_token();
        match t {
            None => false,
            Some(s) => RESERVED_WORDS.contains(&s),
        }
    }
}
