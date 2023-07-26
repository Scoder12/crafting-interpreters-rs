use crate::lexer;
use crate::types::SyntaxKind;
use crate::types::TokenKind;

// Some boilerplate is needed, as rowan settled on using its own
// `struct SyntaxKind(u16)` internally, instead of accepting the
// user's `enum SyntaxKind` as a type parameter.
//
// First, to easily pass the enum variants into rowan via `.into()`:
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(value: SyntaxKind) -> Self {
        Self(value as u16)
    }
}

// Second, implementing the `Language` trait teaches rowan to convert between
// these two SyntaxKind types, allowing for a nicer SyntaxNode API where
// "kinds" are values from our `enum SyntaxKind`, instead of plain u16 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::Root as u16);
        // this is safe because a u16 is bounded on the low end by 0 and on the
        //  high end by the assert
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

// GreenNode is an immutable tree, which is cheap to change,
// but doesn't contain offsets and parent pointers.
use rowan::GreenNode;

// You can construct GreenNodes by hand, but a builder
// is helpful for top-down parsers: it maintains a stack
// of currently in-progress nodes
use rowan::GreenNodeBuilder;

// The parse results are stored as a "green tree".
pub struct Parse {
    green_node: GreenNode,

    #[allow(unused)]
    pub errors: Vec<String>,
}

pub fn parse(mut tokens: Vec<lexer::Token>) -> Parse {
    struct Parser {
        /// input tokens, including whitespace,
        /// in *reverse* order.
        tokens: Vec<lexer::Token>,
        /// the in-progress tree.
        builder: GreenNodeBuilder<'static>,
        /// the list of syntax errors we've accumulated
        /// so far.
        errors: Vec<String>,
    }

    impl Parser {
        /// Advance one token, adding it to the current branch of the tree builder.
        fn bump(&mut self) {
            let tok = self.tokens.pop().unwrap();
            self.builder
                .token(SyntaxKind::from(tok.kind).into(), tok.text.as_str());
        }
        /// Peek at the first unprocessed token
        fn current(&self) -> Option<TokenKind> {
            self.tokens.last().map(|t| t.kind)
        }
        fn skip_ws(&mut self) {
            while self.current() == Some(TokenKind::Whitespace) {
                self.bump()
            }
        }

        fn unexpected(&mut self) {
            self.builder.start_node(SyntaxKind::ErrorUnexpected.into());
            self.errors.push("Unexpected token".into());
            self.bump();
            self.builder.finish_node();
        }
        fn unexpected_eof(&mut self) {
            self.errors.push("Unexpected EOF".into());
        }

        fn primary(&mut self) {
            self.skip_ws();

            match self.current() {
                Some(
                    TokenKind::False
                    | TokenKind::True
                    | TokenKind::Nil
                    | TokenKind::Number
                    | TokenKind::StringLiteral,
                ) => {
                    self.bump();
                }
                Some(TokenKind::LParen) => {
                    self.bump();
                    self.expression();
                    match self.current() {
                        Some(TokenKind::RParen) => self.bump(),
                        Some(_) => self.unexpected(),
                        None => self.unexpected_eof(),
                    }
                }
                Some(_) => self.unexpected(),
                None => self.unexpected_eof(),
            }
        }

        fn unary(&mut self) {
            self.skip_ws();

            if matches!(self.current(), Some(TokenKind::Bang | TokenKind::Minus)) {
                self.builder.start_node(SyntaxKind::Unary.into());
                self.bump();
                self.unary();
                return;
            }

            self.primary();
        }

        fn factor(&mut self) {
            self.builder.start_node(SyntaxKind::Factor.into());
            self.unary();

            while matches!(self.current(), Some(TokenKind::Slash | TokenKind::Star)) {
                self.bump();
                self.unary();
            }
            self.builder.finish_node();
        }

        fn term(&mut self) {
            self.builder.start_node(SyntaxKind::Term.into());
            self.factor();

            while matches!(self.current(), Some(TokenKind::Minus | TokenKind::Plus)) {
                self.bump();
                self.factor();
            }
            self.builder.finish_node();
        }

        fn comparison(&mut self) {
            self.builder.start_node(SyntaxKind::Comparison.into());
            self.term();

            while matches!(
                self.current(),
                Some(
                    TokenKind::Greater
                        | TokenKind::GreaterEqual
                        | TokenKind::Less
                        | TokenKind::LessEqual
                )
            ) {
                self.bump();
                self.term();
            }
            self.builder.finish_node();
        }

        fn equality(&mut self) {
            self.builder.start_node(SyntaxKind::Equality.into());
            self.comparison();

            self.skip_ws();
            while matches!(
                self.current(),
                Some(TokenKind::BangEqual | TokenKind::EqualEqual)
            ) {
                self.bump();
                self.comparison();
            }
            self.builder.finish_node();
        }

        fn expression(&mut self) {
            self.skip_ws();
            self.equality();
        }

        fn parse(mut self) -> Parse {
            self.builder.start_node(SyntaxKind::Root.into());
            self.expression();
            self.skip_ws();

            while self.current() == Some(TokenKind::Newline) {
                self.bump();
            }

            if self.current().is_some() {
                self.builder.start_node(SyntaxKind::ErrorUnexpected.into());
                while self.current().is_some() {
                    self.bump()
                }
                self.errors.push("Expected EOF".to_string());
                self.builder.finish_node();
            }
            self.builder.finish_node();

            Parse {
                green_node: self.builder.finish(),
                errors: self.errors,
            }
        }
    }

    tokens.reverse();
    Parser {
        tokens,
        builder: GreenNodeBuilder::new(),
        errors: Vec::new(),
    }
    .parse()
}

type SyntaxNode = rowan::SyntaxNode<Lang>;
#[allow(unused)]
type SyntaxToken = rowan::SyntaxToken<Lang>;
#[allow(unused)]
type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }
}
