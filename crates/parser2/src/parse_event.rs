use crate::lexer::Token;
use stdx::TextRange;

/// `Parser` produces a flat list of `Event`s.
/// They are converted to a tree-structure in
/// a separate pass, via `TreeBuilder`.
#[derive(Debug)]
pub enum ParseEvent {
    /// This event signifies the start of the node.
    /// It should be either abandoned (in which case the
    /// `kind` is `Tombstone`, and the event is ignored),
    /// or completed via a `Finish` event.
    ///
    /// All tokens between a `Start` and a `Finish` would
    /// become the children of the respective node.
    ///
    /// For left-recursive syntactic constructs, the parser produces
    /// a child node before it sees a parent. `forward_parent`
    /// saves the position of current event's parent.
    ///
    /// Consider this path
    ///
    /// foo::bar
    ///
    /// The events for it would look like this:
    ///
    /// ```text
    /// START(PATH) IDENT('foo') FINISH START(PATH) T![::] IDENT('bar') FINISH
    ///       |                          /\
    ///       |                          |
    ///       +------forward-parent------+
    /// ```
    ///
    /// And the tree would look like this
    ///
    /// ```text
    ///    +--PATH---------+
    ///    |   |           |
    ///    |   |           |
    ///    |  '::'       'bar'
    ///    |
    ///   PATH
    ///    |
    ///   'foo'
    /// ```
    ///
    /// See also `CompletedMarker::precede`.
    Start {
        kind: Token,
        range: TextRange,
        forward_parent: Option<u32>,
    },

    /// Complete the previous `Start` event
    Finish,

    /// Produce a single leaf-element.
    Token(Token, TextRange),

    Error(String, TextRange),
}


impl ParseEvent {
    pub fn tombstone() -> Self {
        ParseEvent::Start {
            kind: Token::Tombstone,
            range: TextRange::new(0,0),
            forward_parent: None,
        }
    }
    pub(crate) fn index(&self) -> u32 {
        match self {
            ParseEvent::Start { .. } => 0,
            ParseEvent::Finish => 1,
            ParseEvent::Token { .. } => 2,
            ParseEvent::Error { .. } => 3,
        }
    }
}
