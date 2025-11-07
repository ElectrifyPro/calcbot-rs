use crate::error::Error;
use paste::paste;
use std::{iter::Peekable, str::CharIndices};

/// Parse the given input string into a type. If there is still input left over after parsing,
/// [`Error::TooManyArguments`] is returned.
pub fn parse_args_full<'a, T: Parse<'a>>(s: &'a str) -> Result<T, Error> {
    let mut parser = Parser::new(s);
    let result = T::parse(&mut parser)?;
    if parser.byte_idx() < s.len() {
        return Err(Error::TooManyArguments);
    }
    Ok(result)
}

/// A high-level parser that parses commands arguments without any heap allocation. To parse a
/// command's arguments, you can use the helper [`parse_args_full`] function, which creates a new
/// [`Parser`] and invokes [`Parse::parse`] on the types you provide.
#[derive(Clone)]
pub struct Parser<'a> {
    /// The raw input string. 
    raw: &'a str,

    /// An iterator over the characters in the string that yields `(byte index, char)` pairs.
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Parser<'a> {
    /// Create a new [`Parser`] from the given input string.
    pub fn new(raw: &'a str) -> Self {
        Self {
            raw,
            iter: raw.char_indices().peekable(),
        }
    }

    /// Returns the byte index of the next character to be parsed, or the length of the string if
    /// the end of the string is reached.
    pub fn byte_idx(&self) -> usize {
        self.iter.clone().peek().map_or(self.raw.len(), |(idx, _)| *idx)
    }

    /// Return the character pointed to by [`Parser::byte_idx`] and advance the parser to the next
    /// character.
    ///
    /// Returns [`None`] if the end of the string is reached.
    pub fn next_char(&mut self) -> Option<char> {
        self.iter.next().map(|(_, c)| c)
    }

    /// Advances the cursor past whitespace characters to the next non-whitespace character, or to the
    /// end of the string (out of bounds).
    pub fn advance_past_whitespace(&mut self) {
        while let Some((_, c)) = self.iter.peek() {
            if c.is_whitespace() {
                // move the byte index forward
                self.next_char();
            } else {
                break;
            }
        }
    }
}

/// A type that can be parsed from a string. Similar to [`FromStr`], but it doesn't require the
/// type to consume all of the input string.
pub trait Parse<'a>: Sized {
    /// Parse the type from a string, and advance the parser past the consumed characters.
    fn parse(parser: &mut Parser<'a>) -> Result<Self, Error>;
}

/// Parse an [`Option<T>`] from a string.
///
/// This works by first checking if the string is empty; if so, it returns [`None`]. Otherwise, it
/// attempts [`Parse::parse`] on the inner type and forwards the result to the caller.
impl<'a, T: Parse<'a>> Parse<'a> for Option<T> {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
        if parser.byte_idx() >= parser.raw.len() {
            // if the string is empty, return None
            // TODO: perhaps check for whitespace first?
            return Ok(None);
        }

        T::parse(parser).map(Some)
    }
}

/// Helper macro that implements [`Parse`] for tuples of [`Parse`] types.
macro_rules! impl_parse_tuple {
    ($($name:ident),+) => {
        impl<'a, $($name: Parse<'a>),+> Parse<'a> for ($($name,)+) {
            fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
                $(
                    paste! {
                        #[allow(non_snake_case)]
                        let [<val_ $name>] = $name::parse(parser)?;
                    }
                    // skip whitespace in between each type
                    parser.advance_past_whitespace();
                )+

                Ok(($(paste! { [<val_ $name>] },)+))
            }
        }
    };
}

impl_parse_tuple!(A, B);
impl_parse_tuple!(A, B, C);
impl_parse_tuple!(A, B, C, D);

/// Helper macro that implements [`Parse`] for integers.
macro_rules! impl_parse_int {
    ($($name:ident),+) => {
        $(
            impl<'a> Parse<'a> for $name {
                fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
                    // clone to emulate peeking more than one character
                    let mut clone = parser.clone();
                    let first_whitespace_idx = clone
                        .iter
                        .find_map(|(byte_idx, c)| c.is_whitespace().then_some(byte_idx))
                        .unwrap_or(parser.raw.len());
                    if first_whitespace_idx == parser.byte_idx() {
                        // no characters to parse
                        return Err(Error::NoArgument);
                    }

                    let str_to_parse = &parser.raw[parser.byte_idx()..first_whitespace_idx];
                    let out = str_to_parse
                        .parse()
                        .map_err(|_| Error::String(format!(
                            "**Couldn't convert `{}` to an integer.** Please provide a valid integer in the range `{}` to `{}`.",
                            str_to_parse,
                            $name::MIN,
                            $name::MAX,
                        )))?;
                    *parser = clone;
                    Ok(out)
                }
            }
        )+
    };
}

impl_parse_int!(u32, usize);

impl<'a> Parse<'a> for f64 {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
        // clone to emulate peeking more than one character
        let mut clone = parser.clone();
        let first_whitespace_idx = clone
            .iter
            .find_map(|(byte_idx, c)| c.is_whitespace().then_some(byte_idx))
            .unwrap_or(parser.raw.len());
        if first_whitespace_idx == parser.byte_idx() {
            // no characters to parse
            return Err(Error::NoArgument);
        }

        let str_to_parse = &parser.raw[parser.byte_idx()..first_whitespace_idx];
        let out = str_to_parse
            .parse()
            .map_err(|_| Error::String(format!(
                "**Couldn't convert `{}` to a number.** Please provide a valid number.",
                str_to_parse,
            )))?;
        *parser = clone;
        Ok(out)
    }
}

/// A string of one or more continuous characters that contain no whitespace.
pub struct Word<'a>(pub &'a str);

impl<'a> Parse<'a> for Word<'a> {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
        let start_byte_idx = parser.byte_idx();
        while let Some((byte_idx, c)) = parser.iter.peek() {
            if c.is_whitespace() {
                // stop parsing when we reach whitespace
                return Ok(Word(&parser.raw[start_byte_idx..*byte_idx]));
            } else {
                // move the byte index forward
                parser.next_char();
            }
        }

        // if we reach the end of the string, return the last word
        if parser.byte_idx() > start_byte_idx {
            Ok(Word(&parser.raw[start_byte_idx..parser.byte_idx()]))
        } else {
            // didn't find any characters
            Err(Error::NoArgument)
        }
    }
}

/// Parses a sequence of space-separated [`Parse`] types.
impl<'a, T: Parse<'a>> Parse<'a> for Vec<T> {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
        let mut out = vec![];
        loop {
            if let Ok(item) = T::parse(parser) {
                out.push(item);
                parser.advance_past_whitespace();
            } else {
                break;
            }
        }
        Ok(out)
    }
}

/// Gets the rest of the string after the current byte index.
pub struct Remainder<'a>(pub &'a str);

impl<'a> Parse<'a> for Remainder<'a> {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
        let start_byte_idx = parser.byte_idx();
        // NOTE: must consume the rest of the string in order to avoid TooManyArguments error
        while parser.next_char().is_some() {}
        Ok(Remainder(parser.raw[start_byte_idx..].trim()))
    }
}
