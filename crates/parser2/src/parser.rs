use drop_bomb::DropBomb;
use std::{borrow::Borrow, rc::Rc};
use stdx::prelude::*;

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
}

pub struct Parser {
    pub(crate) ast_root: Option<RcNode>,
    // pub(crate) change: ChangedRange,
    pub(crate) prior_ast_node: Option<RcNode>,
    pub(crate) next_ast_node: Option<RcNode>,
    pub(crate) tokens: TokenVec,

    pub(crate) possible_states: Vec<ParserState>,
    pub(crate) current_state: ParserState,
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
            possible_states: vec![ParserState::SourceFile],
            prior_ast_node: None,
            next_ast_node: None,
            tokens,
            result: vec![],
            current_state: ParserState::__UNSET,
        }
    }

    pub fn new(text: Rope, ast_root: RcNode, change: &ChangedRange) -> Result<Self> {
        let (possible_states, tokens, prior_ast_node, next_ast_node) = match &change {
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
                    .map_or(String::new(), |n| n.text(&text))
                    + new_text
                    + &next_ast_node
                        .as_ref()
                        .map_or(String::new(), |n| n.text(&text));
                let tokens = lexer::lex_string(
                    &full_change,
                    prior_ast_node.as_ref().map_or(0u32, |n| n.range.start),
                );
                let parser_state = prior_ast_node
                    .as_ref()
                    .map_or(ParserState::OptionalTopLevelStatements, |n| {
                        Self::parser_state_from(n)
                    });

                (vec![parser_state], tokens, prior_ast_node, next_ast_node)
            }
            ChangedRange::Delete { range } => todo!(),
            ChangedRange::Update { range, new_text } => todo!(),
        };

        Ok(Parser {
            ast_root: Some(ast_root),
            possible_states,
            prior_ast_node,
            next_ast_node,
            tokens,
            result: vec![],
            current_state: ParserState::__UNSET,
        })
    }

    pub fn parse(mut self) -> Vec<ParseEvent> {
        // TODO evaluate all possible_states and choose best parse
        self.current_state = self.possible_states[0];
        match self.current_state {
            ParserState::SourceFile => SourceFileRule {}.parse(&mut self),
            ParserState::OptionalTopLevelStatements => {
                todo!()
                // OptionalTopLevelStatementsRule {}.parse(self)
            }
            ParserState::__UNSET => unreachable!(),
        };
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
        self.result.push(ParseEvent::Token(*t,*r));
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

    pub fn parser_state_from(node: &RcNode) -> ParserState {
        todo!()
    }
}
