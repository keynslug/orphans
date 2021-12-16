//! Wildcard parsing and matching.

use std::vec::Vec;
use std::boxed::Box;
// use std::fmt;
// use std::fmt::{Display, Formatter, Write};

/// Any possible production of a wildcard grammar.
///
/// Actually it's a bit wider than required, just to experiment with what kind
/// of expressiveness we can get.
#[derive(Debug, PartialEq)]
enum Production<'a> {
    Sequence(&'a str),
    ManyOf(Vec<Choice<'a>>),
    OneOf(Vec<Choice<'a>>),
    Not(Box<Production<'a>>)
}

/// Alphabet of a non-terminal grammar production.
///
/// It's either _any character from this slice_ or _any character from this
/// range_.
#[derive(Debug, PartialEq)]
enum Choice<'a> {
    AnyOf(&'a str),
    Range(char, char)
}

/// Conceptually, `Wildcard` is some grammar in language of all strings.
#[derive(Debug, PartialEq)]
pub struct Wildcard<'a> (
    Vec<Production<'a>>
);

impl<'a> Wildcard<'a> {
    /// Creates an empty `Wildcard`.
    pub fn new() -> Self { Self(Vec::<_>::new()) }
}

/// Errors which can occur during parsing string as a wildcard.
#[derive(Debug, PartialEq)]
pub enum WildcardParseError {
    Incomplete,
    InvalidCharRange(char, char)
}

/// Type to hold some state during parsing string as a wildcard.
#[derive(Debug)]
struct WildcardParser<'a> {
    source: &'a str,
    result: Wildcard<'a>,
    /// Where has a capture started?
    start: Option<usize>,
    /// Should we negate next token?
    negate: bool
}

impl<'a> WildcardParser<'a> {

    fn new(source: &'a str) -> WildcardParser<'a> {
        Self {
            source,
            result: Wildcard::new(),
            start: None,
            negate: false
        }
    }

    /// Proceeds with parsing.
    ///
    /// Spits out a `Grammar` if string is valid wildcard representation,
    /// or some `GrammarParseError` otherwise.
    fn run(&mut self) -> Result<Wildcard<'a>, WildcardParseError> {
        use Production::*;
        use Choice::*;
        use WildcardParseError::*;
        let mut token = Sequence(self.source);
        let mut iter = self.source.char_indices();
        let mut prev: (usize, char) = Default::default();
        while let Some((index, c)) = iter.next() {
            if self.start.is_none() {
                self.start = Some(index);
            }
            match c {
                '*' =>
                    if let Sequence(_) = &token {
                        self.flush(index);
                        self.reset_capture();
                        self.push(ManyOf(Vec::new()));
                    },

                '?' =>
                    if let Sequence(_) = &token {
                        self.flush(index);
                        self.reset_capture();
                        self.push(OneOf(Vec::new()));
                    },

                '[' =>
                    if let Sequence(_) = &token {
                        self.flush(index);
                        token = OneOf(Vec::new());
                        let (index, c) = iter.next().ok_or(Incomplete)?;
                        self.start_capture(index);
                        if '!' == c {
                            self.negate = true;
                            self.reset_capture();
                        }
                    },

                '-' => {
                    if let OneOf(choices) = &mut token {
                        if let Some(capture) = self.capture(prev.0) {
                            choices.push(AnyOf(capture));
                        }
                        let (_, c) = iter.next().ok_or(Incomplete)?;
                        if c < prev.1 {
                            return Err(InvalidCharRange(prev.1, c))
                        }
                        self.reset_capture();
                        choices.push(Range(prev.1, c));
                    }
                }

                ']' =>
                    if let OneOf(choices) = &mut token {
                        if let Some(capture) = self.capture(index) {
                            choices.push(AnyOf(capture));
                        }
                        let token = std::mem::replace(&mut token, Sequence(self.source));
                        self.reset_capture();
                        self.push(token);
                    }

                _ =>
                    ()

            }
            prev = (index, c);
        }
        if let Sequence(_) = &token {
            self.flush(self.source.len());
            let result = std::mem::replace(&mut self.result, Wildcard::new());
            Ok(result)
        }
        else {
            Err(WildcardParseError::Incomplete)
        }
    }

    /// Push a `Sequence` to the buffer if there's non-empty active capture.
    fn flush(&mut self, index: usize) {
        if let Some(capture) = self.capture(index) {
            self.push(Production::Sequence(capture));
        }
    }

    /// Push a given token to the buffer, negate if needed.
    fn push(&mut self, p: Production<'a>) {
        self.result.0.push(
            if self.negate {
                self.negate = false;
                Production::Not(Box::new(p))
            }
            else {
                p
            }
        );
    }

    /// Clears active capture index.
    fn reset_capture(&mut self) {
        self.start = None;
    }

    /// Starts active capture from `index`.
    fn start_capture(&mut self, index: usize) {
        self.start = Some(index);
    }

    /// Cuts a slice with the active capture if it's non-empty.
    fn capture(&mut self, index: usize) -> Option<&'a str> {
        if let Some(start) = self.start {
            if index > start {
                return Some(&self.source[start .. index])
            }
        }
        None
    }

}

impl std::fmt::Display for Production<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::{Formatter, Result};
        use Production::*;
        fn run(p: &Production<'_>, f: &mut Formatter, negate: bool) -> Result {
            match self {
                Sequence(s) => write!(f, "{}", s),
                ManyOf(_) => write!(f, "*"),
                OneOf(choices) if choices.is_empty() => write!(f, "?"),
                OneOf(choices) =>
                    {
                        write!(f, "[");
                        if negate { write!(f, "!") };
                        choices
                            .iter()
                            .try_for_each(|c| c.fmt(f))?;
                        fmt.write_char(']')?;
                        Ok(())
                    },
                Not(token) =>
                    fmt_token(token, true, fmt)
            }
        }
        match self {
            Sequence(s) => s.fmt(f),
            ManyOf(_) => '*'.fmt(f),
            OneOf(choices) if choices.is_empty() => '?'.fmt(f),
            OneOf(choices) =>
                {
                    fmt.write_char('[')?;
                    if negate {
                        fmt.write_char('!')?;
                    }
                    choices
                        .iter()
                        .try_for_each(|c| c.fmt(f))?;
                    fmt.write_char(']')?;
                    Ok(())
                },
            Not(token) =>
                fmt_token(token, true, fmt)
        }
    }
}

impl std::fmt::Display for Wildcard<'_> {

    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::{Formatter, Result};
        fn fmt_choice(choice: &Choice<'_>, fmt: &mut Formatter) -> Result {
            match choice {
                Choice::AnyOf(s) =>
                    fmt.write_str(s),
                Choice::Range(from, to) =>
                    [*from, '-', *to]
                        .iter()
                        .try_for_each(|&c| fmt.write_char(c))
            }
        }
        fn fmt_token(token: &Token<'_>, negate: bool, fmt: &mut Formatter) -> Result {
            match token {
                Token::Sequence(s) =>
                    fmt.write_str(s),
                Token::ManyOf(_) =>
                    fmt.write_char('*'),
                Token::OneOf(choices) if choices.is_empty() =>
                    fmt.write_char('?'),
                Token::OneOf(choices) =>
                    {
                        fmt.write_char('[')?;
                        if negate {
                            fmt.write_char('!')?;
                        }
                        choices
                            .iter()
                            .try_for_each(|c| fmt_choice(c, fmt))?;
                        fmt.write_char(']')?;
                        Ok(())
                    },
                Token::Not(token) =>
                    fmt_token(token, true, fmt)
            }
        }
        self.0
            .iter()
            .try_for_each(|t| fmt_token(t, false, fmt))
    }

}

// impl<'a> Wildcard<'a> {

//     pub fn parse(source: &'a str) -> Result<Wildcard<'a>, WildcardParseError> {
//         WildcardParser::new(source).run().map(|grammar| Self(grammar))
//     }

//     /// Checks whether subject satisfies wildcard.
//     pub fn matches(&self, subject: &str) -> bool {
//         let iter = self.0.iter();
//         self.matches_after(iter, subject)
//     }

//     fn matches_after<I>(&self, iter: I, subject: &str) -> bool where
//         I: Iterator<Item = &'a Token<'a>>
//     {
//         false
//     }

// }
