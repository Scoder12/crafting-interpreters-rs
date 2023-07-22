#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // single character
    LParen,
    RParen,
    LBrac,
    RBrac,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    StringLiteral, // don't clobber builtin "String"
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    LineComment,
    BlockComment,
    Whitespace,
    Newline,

    ErrorUnexpected,
    ErrorUnterminatedString,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    kind: TokenKind,
    text: String,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut res = Vec::new();
    let mut rest = input;
    while !rest.is_empty() {
        let next = valid_token(rest).unwrap_or_else(|| invalid_token(rest));
        rest = &rest[next.text.len()..];
        res.push(next);
    }
    debug_assert!(
        res.iter().map(|t| t.text.clone()).collect::<String>() == input,
        "expected lex result to cleanly match input"
    );
    res
}

static KEYWORDS: phf::Map<&'static str, TokenKind> = phf::phf_map! {
    "and" => TokenKind::And,
    "class" => TokenKind::Class,
    "else" => TokenKind::Else,
    "false" => TokenKind::False,
    "for" => TokenKind::For,
    "fn" => TokenKind::Fn,
    "if" => TokenKind::If,
    "nil" => TokenKind::Nil,
    "or" => TokenKind::Or,
    "print" => TokenKind::Print,
    "return" => TokenKind::Return,
    "super" => TokenKind::Super,
    "this" => TokenKind::This,
    "true" => TokenKind::True,
    "var" => TokenKind::Var,
    "while" => TokenKind::While,
};

fn valid_token(input: &str) -> Option<Token> {
    if input.is_empty() {
        return None;
    }

    let mut chars = input.chars().peekable();
    let first_char = chars.next().unwrap();
    macro_rules! t1 {
        ($kind: expr) => {
            return Some(Token {
                kind: $kind,
                text: first_char.into(),
            })
        };
    }
    macro_rules! t2 {
        ($kind: expr) => {
            return Some(Token {
                kind: $kind,
                text: input.chars().take(2).collect(),
            })
        };
    }
    macro_rules! peek_t2 {
        ($c: expr, $kind: expr) => {
            if let Some($c) = chars.peek() {
                t2!($kind);
            }
        };
    }

    use TokenKind::*;
    match first_char {
        '(' => t1!(LParen),
        ')' => t1!(RParen),
        '{' => t1!(LBrac),
        '}' => t1!(RBrac),
        ',' => t1!(Comma),
        '.' => t1!(Dot),
        '-' => t1!(Minus),
        '+' => t1!(Plus),
        ';' => t1!(Semicolon),
        '*' => t1!(Star),
        '!' => {
            peek_t2!('=', BangEqual);
            t1!(Bang);
        }
        '=' => {
            peek_t2!('=', EqualEqual);
            t1!(Equal);
        }
        '<' => {
            peek_t2!('=', LessEqual);
            t1!(Less);
        }
        '>' => {
            peek_t2!('=', GreaterEqual);
            t1!(Greater);
        }
        '/' => {
            if let Some('/') = chars.peek() {
                let text = std::iter::once(first_char)
                    .chain(std::iter::once(chars.next().unwrap()))
                    .chain(chars.take_while(|c| *c != '\n'))
                    .collect();
                return Some(Token {
                    kind: TokenKind::LineComment,
                    text,
                });
            }
            if let Some('*') = chars.peek() {
                let iter1 = std::iter::once((first_char, *chars.peek().unwrap()))
                    .chain(chars.zip(input.chars().skip(2)))
                    .take_while(|(c, n)| *c != '*' || *n != '/')
                    .map(|(_c, n)| n);
                let text = std::iter::once(first_char)
                    .chain(iter1)
                    .chain(std::iter::once('/'))
                    .collect();
                return Some(Token {
                    kind: BlockComment,
                    text,
                });
            }
            t1!(Slash);
        }
        ' ' | '\r' | '\t' => {
            let text = std::iter::once(first_char)
                .chain(chars.take_while(|c| *c == ' ' || *c == '\r' || *c == '\t'))
                .collect();
            return Some(Token {
                kind: TokenKind::Whitespace,
                text,
            });
        }
        '"' => {
            let mut text = String::new();
            text.push('"');
            loop {
                let Some(c) = chars.next() else {
                    return Some(Token { kind: TokenKind::ErrorUnterminatedString, text });
                };
                text.push(c);
                if c == '"' {
                    break;
                }
            }
            return Some(Token {
                kind: TokenKind::StringLiteral,
                text,
            });
        }
        '\n' => t1!(Newline),
        c if c.is_numeric() => {
            let mut text = String::new();
            text.push(c);
            let mut c: Option<char>;
            loop {
                c = chars.next();
                let Some(c) = c else {
                    break;
                };
                if !c.is_numeric() && c != '_' {
                    break;
                }
                text.push(c);
            }

            if c.is_some()
                && c.unwrap() == '.'
                && chars.peek().map(|c| c.is_numeric()).unwrap_or(false)
            {
                text.push('.');
                for c in chars.by_ref() {
                    if !c.is_numeric() && c != '_' {
                        break;
                    }
                    text.push(c);
                }
            }
            return Some(Token {
                kind: TokenKind::Number,
                text,
            });
        }
        c @ '_' | c if c.is_alphabetic() => {
            let ident: String = std::iter::once(c)
                .chain(chars.take_while(|c| c.is_alphabetic() || c.is_numeric() || *c == '_'))
                .collect();
            if let Some(kind) = KEYWORDS.get(&ident) {
                return Some(Token {
                    kind: *kind,
                    text: ident,
                });
            }
            return Some(Token {
                kind: Identifier,
                text: ident,
            });
        }
        _ => {}
    }
    None
}

fn invalid_token(input: &str) -> Token {
    let mut len = 0;
    for c in input.chars() {
        len += c.len_utf8();
        if valid_token(&input[len..]).is_some() {
            break;
        }
    }
    Token {
        kind: TokenKind::ErrorUnexpected,
        text: input[..len].into(),
    }
}
