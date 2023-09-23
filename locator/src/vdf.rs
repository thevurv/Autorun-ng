use nom::{
	branch::alt,
	bytes::complete::{escaped_transform, take_while},
	character::complete::{char, multispace0, multispace1, one_of},
	combinator::map,
	error::{ContextError, ParseError},
	multi::separated_list0,
	sequence::{delimited, preceded, separated_pair, terminated},
	IResult,
};

#[derive(Debug, PartialEq)]
pub enum Value {
	Str(String),
	Object(Vec<(String, Value)>),
}

fn non_quote<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
	take_while(|c| c != '"')(i)
}

fn string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, String, E> {
	delimited(
		char('"'),
		escaped_transform(
			non_quote,
			'\\',
			alt((
				nom::combinator::value('\\', char('\\')),
				nom::combinator::value('\n', char('n')),
				nom::combinator::value('"', char('"'))
			)),
		),
		char('"'),
	)(i)
}

#[test]
fn test_string() {
	assert!(string::<nom::error::Error<&str>>(r#""""#).is_ok());
	assert!(string::<nom::error::Error<&str>>(r#""a""#).is_ok());

	assert_eq!(
		string::<nom::error::Error<&str>>(r#""Hello, world!""#),
		Ok(("", String::from("Hello, world!")))
	);
	assert_eq!(
		string::<nom::error::Error<&str>>(r#""Hello, \\world!""#),
		Ok(("", String::from("Hello, \\world!")))
	);
}

fn keyvalue<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, (String, Value), E> {
	preceded(multispace0, separated_pair(string, multispace1, value))(i)
}

#[test]
fn test_keyvalue() {
	assert_eq!(
		keyvalue::<nom::error::Error<&str>>(r#""0"    "1""#),
		Ok(("", (String::from("0"), Value::Str(String::from("1")))))
	);
	assert_eq!(
		keyvalue::<nom::error::Error<&str>>(r#""" """#),
		Ok(("", (String::new(), Value::Str(String::new()))))
	);
	assert_eq!(
		keyvalue::<nom::error::Error<&str>>(r#""foo" {}"#),
		Ok(("", (String::from("foo"), Value::Object(vec![]))))
	);
}

fn keyvalues<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, Vec<(String, Value)>, E> {
	separated_list0(multispace1, keyvalue)(i)
}

#[test]
fn test_keyvalues() {
	assert_eq!(
		keyvalues::<nom::error::Error<&str>>("\"foo\" {}\n\"bar\" \"12\""),
		Ok((
			"",
			vec![(String::from("foo"), Value::Object(vec![])), (String::from("bar"), Value::Str(String::from("12")))]
		))
	);
}

fn object<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, Vec<(String, Value)>, E> {
	delimited(
		terminated(char('{'), multispace0),
		keyvalues,
		preceded(multispace0, char('}')),
	)(i)
}

#[test]
fn test_object() {
	assert_eq!(
		object::<nom::error::Error<&str>>("{ \"foo\" {}\n\"bar\" \"12\" }"),
		Ok((
			"",
			vec![(String::from("foo"), Value::Object(vec![])), (String::from("bar"), Value::Str(String::from("12")))]
		))
	);
}

/// here, we apply the space parser before trying to parse a value
fn value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
	i: &'a str,
) -> IResult<&'a str, Value, E> {
	preceded(
		multispace0,
		alt((map(object, Value::Object), map(string, Value::Str))),
	)(i)
}

pub fn parse<'a>(i: &'a str) -> IResult<&'a str, (String, Value), nom::error::Error<&'a str>> {
	keyvalue(i)
}
