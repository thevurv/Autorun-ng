#[derive(Debug, PartialEq)]
pub enum Value<'src> {
	String(&'src [u8]),
	Object(Vec<(&'src [u8], Value<'src>)>),
}

impl core::fmt::Display for Value<'_> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Value::String(s) => write!(f, "\"{}\"", String::from_utf8_lossy(s)),
			Value::Object(fields) => {
				write!(f, "{{")?;
				for (i, (key, value)) in fields.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "\"{}\": {}", String::from_utf8_lossy(key), value)?;
				}
				write!(f, "}}")
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Token<'src> {
	LeftCurly,
	RightCurly,
	String(&'src [u8]),
}

#[derive(Debug, thiserror::Error)]
pub enum TokenizeError {
	#[error("Unexpected character")]
	UnexpectedCharacter(char),

	#[error("Unexpected end of input")]
	UnexpectedEndOfInput,
}

pub fn tokenize(src: &[u8]) -> Result<Vec<Token<'_>>, TokenizeError> {
	let mut tokens = vec![];

	let mut ptr = 0;
	while ptr < src.len() {
		match src[ptr] {
			b'{' => {
				ptr += 1;
				tokens.push(Token::LeftCurly);
			}

			b'}' => {
				ptr += 1;
				tokens.push(Token::RightCurly);
			}

			b'"' => {
				ptr += 1;

				let start = ptr;
				loop {
					if ptr == src.len() {
						return Err(TokenizeError::UnexpectedEndOfInput);
					}

					if src[ptr] == b'"' {
						break;
					}

					ptr += 1;
				}

				tokens.push(Token::String(&src[start..ptr]));
				ptr += 1;
			}

			b' ' | b'\t' | b'\n' | b'\r' => {
				ptr += 1;
			}

			_ => {
				return Err(TokenizeError::UnexpectedCharacter(src[ptr] as char));
			}
		}
	}

	Ok(tokens)
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	#[error("Unexpected end of input")]
	UnexpectedEndOfInput(usize),
}

pub fn parse<'src>(tokens: &'src [Token<'src>]) -> Result<(&'src [u8], Value<'src>), ParseError> {
	let mut i = 0;
	match parse_kv_pair(tokens, &mut i)? {
		Some(kv_pair) => Ok(kv_pair),
		None => Err(ParseError::UnexpectedEndOfInput(i)),
	}
}

fn parse_string<'src>(tokens: &'src [Token<'src>], i: &mut usize) -> Option<Value<'src>> {
	match tokens.get(*i) {
		Some(Token::String(s)) => {
			*i += 1;
			Some(Value::String(s))
		}
		_ => None,
	}
}

fn parse_object<'src>(tokens: &'src [Token<'src>], i: &mut usize) -> Result<Option<Value<'src>>, ParseError> {
	if *i >= tokens.len() {
		return Ok(None);
	}

	let Token::LeftCurly = tokens[*i] else {
		return Ok(None);
	};

	*i += 1;

	let mut obj = Vec::new();
	while *i < tokens.len() {
		if let Some((k, v)) = parse_kv_pair(tokens, i)? {
			obj.push((k, v));
		} else {
			break;
		}
	}

	if *i >= tokens.len() {
		return Err(ParseError::UnexpectedEndOfInput(*i));
	}

	let Token::RightCurly = tokens[*i] else {
		return Err(ParseError::UnexpectedEndOfInput(*i));
	};

	*i += 1;
	Ok(Some(Value::Object(obj)))
}

fn parse_kv_pair<'src>(tokens: &'src [Token<'src>], i: &mut usize) -> Result<Option<(&'src [u8], Value<'src>)>, ParseError> {
	if *i >= tokens.len() {
		return Ok(None);
	}

	let Token::String(key) = tokens[*i] else {
		return Ok(None);
	};

	*i += 1;

	if let Some(obj) = parse_object(tokens, i)? {
		Ok(Some((key, obj)))
	} else if let Some(str) = parse_string(tokens, i) {
		Ok(Some((key, str)))
	} else {
		Err(ParseError::UnexpectedEndOfInput(*i))
	}
}
