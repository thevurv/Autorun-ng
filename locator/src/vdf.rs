use nom::{
	branch::alt,
	bytes::complete::{take_while, escaped},
	character::complete::{char, one_of, multispace0, multispace1},
	combinator::map,
	error::{ContextError, ParseError},
	multi::separated_list0,
	sequence::{delimited, preceded, separated_pair, terminated},
	IResult
};

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
	Str(&'a str),
	Object(Vec<(&'a str, Value<'a>)>),
}

fn non_quote<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
	take_while(|c| c != '"')(i)
}

fn string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
	delimited(char('"'), escaped(non_quote, '\\', one_of("\"n\\")), char('"'))(i)
}

#[test]
fn test_string() {
	assert!( string::<nom::error::Error<&str>>(r#""""#).is_ok() );
	assert!( string::<nom::error::Error<&str>>(r#""a""#).is_ok() );

	assert_eq!( string::<nom::error::Error<&str>>(r#""Hello, world!""#), Ok(("", "Hello, world!")) );
	assert_eq!( string::<nom::error::Error<&str>>(r#""Hello, \\world!""#), Ok(("", "Hello, \\\\world!")) );
}

fn keyvalue<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, (&'a str, Value), E> {
	preceded(multispace0, separated_pair(string, multispace1, value))(i)
}

#[test]
fn test_keyvalue() {
	assert_eq!( keyvalue::<nom::error::Error<&str>>(r#""0"    "1""#), Ok(("", ("0", Value::Str("1")))) );
	assert_eq!( keyvalue::<nom::error::Error<&str>>(r#""" """#), Ok(("", ("", Value::Str("")))) );
	assert_eq!( keyvalue::<nom::error::Error<&str>>(r#""foo" {}"#), Ok(("", ("foo", Value::Object(vec![])))) );
}

fn keyvalues<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, Vec<(&'a str, Value)>, E> {
	separated_list0(multispace1, keyvalue)(i)
}

#[test]
fn test_keyvalues() {
	assert_eq!( keyvalues::<nom::error::Error<&str>>("\"foo\" {}\n\"bar\" \"12\""), Ok(("", vec![("foo", Value::Object(vec![])), ("bar", Value::Str("12"))])) );
}

fn object<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, Vec<(&str, Value)>, E> {
	delimited(
		terminated(char('{'), multispace0),
		keyvalues,
		preceded(multispace0, char('}')),
	)(i)
}

#[test]
fn test_object() {
	assert_eq!( object::<nom::error::Error<&str>>("{ \"foo\" {}\n\"bar\" \"12\" }"), Ok(("", vec![("foo", Value::Object(vec![])), ("bar", Value::Str("12"))])) );
}

/// here, we apply the space parser before trying to parse a value
fn value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, Value<'a>, E> {
	preceded(
		multispace0,
		alt((map(object, Value::Object), map(string, Value::Str))),
	)(i)
}

pub fn parse<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, Value<'a>, E> {
	map(keyvalues, Value::Object)(i)
}