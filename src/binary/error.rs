use nom::error::Error as NomError;
use nom::IResult;
use std::ops::Sub;

pub type ParseResult<'a, O> = IResult<&'a [u8], O, NomError<&'a [u8]>>;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryError {
    InvalidMagic,
    InvalidVersion,
    UnknownSection,
    InvalidSectionSize,
    UnexpectedEOF,
    TypeMismatch,
    InvalidInstruction,
}

impl std::fmt::Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryError::InvalidMagic => write!(f, "Invalid magic number"),
            BinaryError::InvalidVersion => write!(f, "Invalid version number"),
            BinaryError::UnknownSection => write!(f, "Unknown section"),
            BinaryError::InvalidSectionSize => write!(f, "Invalid section size"),
            BinaryError::UnexpectedEOF => write!(f, "Unexpected end of file"),
            BinaryError::TypeMismatch => write!(f, "Type mismatch"),
            BinaryError::InvalidInstruction => write!(f, "Invalid instruction"),
        }
    }
}

impl std::error::Error for BinaryError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub offset: usize,
    pub length: usize,
}

impl SourceLocation {
    pub fn new(offset: usize, length: usize) -> Self {
        Self { offset, length }
    }

    pub fn from_slice<'a>(base: &'a [u8], slice: &'a [u8]) -> Self {
        let offset = base.len() - slice.len();
        let length = slice.len();
        Self::new(offset, length)
    }

    pub fn end(&self) -> usize {
        self.offset + self.length
    }

    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.offset && offset < self.end()
    }
}

#[derive(Debug, Clone)]
pub struct Located<T> {
    pub value: T,
    pub location: SourceLocation,
}

impl<T> Located<T> {
    pub fn new(value: T, location: SourceLocation) -> Self {
        Self { value, location }
    }

    pub fn from_parse<'a>(base: &'a [u8], value: T, remaining: &'a [u8]) -> Self {
        let offset = base.len() - remaining.len();
        let length = remaining.len();
        Self::new(value, SourceLocation::new(offset, length))
    }

    pub fn map<U, F>(self, f: F) -> Located<U>
    where
        F: FnOnce(T) -> U,
    {
        Located::new(f(self.value), self.location.clone())
    }

    pub fn location(&self) -> &SourceLocation {
        &self.location
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

pub fn with_location<'a, T, P, O>(
    mut parser: P,
) -> impl FnMut(&'a [u8]) -> ParseResult<'a, Located<O>>
where
    P: FnMut(&'a [u8]) -> ParseResult<'a, O>,
{
    move |input: &'a [u8]| -> ParseResult<'a, Located<O>> {
        let base = input;
        let (remaining, value) = parser(input)?;
        let offset = base.len() - remaining.len();
        let location = SourceLocation::new(offset, 0);
        Ok((remaining, Located::new(value, location)))
    }
}

pub fn capture_location<'a, T>(
    parser: impl Fn(&'a [u8]) -> ParseResult<'a, T>,
) -> impl Fn(&'a [u8]) -> ParseResult<'a, Located<T>> {
    move |input: &'a [u8]| -> ParseResult<'a, Located<T>> {
        let base = input;
        let (remaining, value) = parser(input)?;
        let offset = base.len() - remaining.len();
        Ok((
            remaining,
            Located::new(value, SourceLocation::new(offset, 0)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_new() {
        let loc = SourceLocation::new(100, 10);
        assert_eq!(loc.offset, 100);
        assert_eq!(loc.length, 10);
    }

    #[test]
    fn test_source_location_end() {
        let loc = SourceLocation::new(100, 10);
        assert_eq!(loc.end(), 110);
    }

    #[test]
    fn test_source_location_contains() {
        let loc = SourceLocation::new(100, 10);
        assert!(!loc.contains(99));
        assert!(loc.contains(100));
        assert!(loc.contains(105));
        assert!(!loc.contains(110));
    }

    #[test]
    fn test_source_location_from_slice() {
        let base = &[0u8; 100];
        let slice = &base[10..20];
        let loc = SourceLocation::from_slice(base, slice);
        assert_eq!(loc.offset, 10);
        assert_eq!(loc.length, 10);
    }

    #[test]
    fn test_located_new() {
        let loc = SourceLocation::new(100, 10);
        let located = Located::new(42i32, loc);
        assert_eq!(located.value, 42);
        assert_eq!(located.location.offset, 100);
    }

    #[test]
    fn test_located_from_parse() {
        let base = &[0u8, 1, 2, 3, 4];
        let value = "test";
        let remaining = &base[2..];
        let located = Located::from_parse(base, value, remaining);
        assert_eq!(located.value, "test");
        assert_eq!(located.location.offset, 2);
    }

    #[test]
    fn test_located_map() {
        let loc = SourceLocation::new(100, 10);
        let located: Located<i32> = Located::new(42, loc);
        let mapped = located.map(|x| x * 2);
        assert_eq!(mapped.value, 84);
        assert_eq!(mapped.location.offset, 100);
    }
}
