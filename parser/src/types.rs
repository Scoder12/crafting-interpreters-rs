// allows the CST to extend the lex tokens
macro_rules! build_impls {
    ($v:tt, $($values:tt),*) => {
        #[derive(Clone, Debug, Copy, PartialEq, Eq)]
        pub enum TokenKind {
            $v,
            $($values),*
        }

        #[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u16)]
        pub enum SyntaxKind {
            $v = 0, // fix variant will be zero
            $($values),*,

            Root // root must be last; it is used for bounds checking
        }

        impl From<TokenKind> for SyntaxKind {
            fn from(value: TokenKind) -> Self {
                match value {
                    TokenKind::$v => SyntaxKind::$v,
                    $( TokenKind::$values => SyntaxKind::$values ),*
                }
            }
        }
    };
}

build_impls! {
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
    ErrorUnterminatedString
}
