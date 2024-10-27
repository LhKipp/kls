use drop_bomb::DropBomb;
use std::{borrow::Borrow, rc::Rc};
use stdx::prelude::*;
use tap::Tap;

use crate::node_helper::prior_sibling_of;
use crop::Rope;
use stdx::TextRange;

use crate::grammar::*;

use crate::marker::Marker;
use crate::{
    ast_util::descendant_containing_byte,
    lexer::{self, Token, TokenVec},
    node::RcNode,
    parse_event::ParseEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParserState {
    SourceFile,
    OptionalTopLevelStatements,
    __UNSET,

    PackageDeclPackageKeywordParsed,
    PackageDeclPeriodParsed,
    PackageDeclIdentParsed,
}

pub struct Parser {
    pub(crate) ast_root: Option<RcNode>,
    // pub(crate) change: ChangedRange,
    pub(crate) prior_ast_node: Option<RcNode>,
    pub(crate) next_ast_node: Option<RcNode>,
    pub(crate) tokens: TokenVec,

    pub(crate) state: Vec<Box<dyn Rule>>,
    pub(crate) result: Vec<ParseEvent>,
}

pub enum ChangedRange {
    Insert { at_byte: u32, new_text: String },
    Delete { range: TextRange },
    Update { range: TextRange, new_text: String },
}

impl Parser {
    pub fn new_no_state(new_text: &str) -> Self {
        let tokens = lexer::lex_string(new_text, 0);
        Parser {
            ast_root: None,
            prior_ast_node: None,
            next_ast_node: None,
            tokens,
            result: vec![],
            state: Self::default_state(),
        }
    }

    pub fn try_new(text: &Rope, ast_root: RcNode, change: &ChangedRange) -> Result<Self> {
        let (state, tokens, prior_ast_node, next_ast_node) = match &change {
            ChangedRange::Insert { at_byte, new_text } => {
                let prior_ast_node = if *at_byte == 0 {
                    None
                } else {
                    descendant_containing_byte(&ast_root, *at_byte - 1)
                        .ok_logged()
                        .cloned()
                };
                let next_ast_node = descendant_containing_byte(&ast_root, *at_byte)
                    .ok_logged()
                    .cloned();

                let full_change = prior_ast_node
                    .as_ref()
                    .map_or(String::new(), |n| n.text(text))
                    + new_text
                    + &next_ast_node
                        .as_ref()
                        .map_or(String::new(), |n| n.text(text));
                let tokens = lexer::lex_string(
                    &full_change,
                    prior_ast_node.as_ref().map_or(0u32, |n| n.range.start),
                );
                let parser_state = prior_ast_node
                    .as_ref()
                    .map_or_else(Self::default_state, |n| Self::parser_state_from(n.clone()));

                (parser_state, tokens, prior_ast_node, next_ast_node)
            }
            ChangedRange::Delete { range } => todo!(),
            ChangedRange::Update { range, new_text } => todo!(),
        };

        Ok(Parser {
            ast_root: Some(ast_root),
            prior_ast_node,
            next_ast_node,
            tokens,
            result: vec![],
            state,
        })
    }

    pub fn parse(mut self) -> Vec<ParseEvent> {
        // TODO evaluate all possible_states and choose best parse
        // assert_eq!(self.state.len(), 1);

        let next_rule = self.state.pop();
        next_rule.unwrap().parse(&mut self);

        debug!("parse result: {:?}", self.result);
        self.result
    }

    pub(crate) fn start(&mut self, kind: Token, forward_parent: Option<u32>) -> Marker {
        let range = self
            .tokens
            .current()
            .map_or(TextRange::new(0, 0), |(_, r)| *r);
        let idx = self.result.len() as u32;
        self.result.push(ParseEvent::Start {
            kind,
            range,
            forward_parent,
        });
        Marker::new(idx, kind.into())
    }

    pub(crate) fn finish(&mut self) {
        trace!("Finishing node");
        self.result.push(ParseEvent::Finish);
    }

    /// Consume the next token if it is `kind` or emit an error
    /// otherwise.
    pub(crate) fn expect(&mut self, token: Token) -> bool {
        if self.eat(token) {
            trace!("Expected {} and found it", token);
            return true;
        }
        trace!("Expected {} but didn't find it. Creating error.", token);
        self.error(
            format!("expected {}", token),
            self.tokens
                .currently_at_as_range()
                .unwrap_or(TextRange::new(0, 0)),
        );
        false
    }

    pub(crate) fn eat(&mut self, token: Token) -> bool {
        let Some((t, r)) = self.tokens.current() else {
            return false;
        };
        if *t != token {
            trace!("Could not eat for ts {}", token);
            return false;
        }
        trace!("Eating {}", token);
        self.result.push(ParseEvent::Token(*t, *r));
        self.tokens.bump();
        true
    }

    pub(crate) fn at(&self, token: Token) -> bool {
        self.tokens.current().is_some_and(|(t, _)| *t == token)
    }

    pub(crate) fn error(&mut self, err: String, range: TextRange) {
        // debug!(
        //     "Parser error: {:?} (nth_tokens(0,1,2): {} {} {})",
        //     err,
        //     self.nth(0),
        //     self.nth(1),
        //     self.nth(2)
        // );
        self.result.push(ParseEvent::Error(err, range));
    }

    pub fn parser_state_from(mut node: RcNode) -> Vec<BoxedRule> {
        trace!("getting parser state from {}", node.ntype);
        const KNOWN_STATES: [Token; 2] = [Token::PackageDecl, Token::SourceFile];
        let original_node = node.clone();

        let mut state: Vec<BoxedRule> = vec![];

        loop {
            let node_type = if node.ntype == Token::Ws {
                prior_sibling_of(&node).expect("no prior sibling").ntype
            } else {
                node.ntype
            };

            if node_type == Token::SourceFile {
                if state.is_empty() {
                    state.push(Box::new(SourceFileRule {}));
                }
                break;
            }

            // goto parent
            trace!("going to parent of node {}", node.ntype);
            node = node.parent.clone().expect("node has no parent");
            let parent_type = node.ntype;

            trace!("state from node {} and parent {}", node_type, parent_type);
            match parent_type {
                Token::SourceFile => {
                    // Todo add state from node_type
                    state.push(Box::new(SourceFileRule {}));
                }
                Token::PackageDecl => state.push(Box::new(PackageStatement::new(Some(node_type)))),
                _ => todo!("node type not yet handled {}", parent_type),
            };
        }

        state
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .tap(|state| trace!("returning state {:?}", state))
    }

    fn default_state() -> Vec<BoxedRule> {
        vec![Box::new(SourceFileRule {})]
    }
}
