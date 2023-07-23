use crate::types::SyntaxKind;

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
enum Lang {}

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
struct Parse {
    green_node: GreenNode,

    #[allow(unused)]
    errors: Vec<String>,
}

fn parse(text: &str) -> Parse {
    struct Parser {
        /// input tokens, including whitespace,
        /// in *reverse* order.
        tokens: Vec<(SyntaxKind, String)>,
        /// the in-progress tree.
        builder: GreenNodeBuilder<'static>,
        /// the list of syntax errors we've accumulated
        /// so far.
        errors: Vec<String>,
    }

    /// The outcome of parsing a single S-expression
    enum ParseRes {
        Ok,
        /// Nothing was parsed, as no significant tokens remained
        Eof,
    }

    todo!()
}
