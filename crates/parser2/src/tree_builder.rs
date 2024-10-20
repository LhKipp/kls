use crate::Token;
use crop::Rope;
use std::borrow::BorrowMut;
use std::mem;
use std::rc::Rc;
use stdx::TextRange;

use crate::parse_event::ParseEvent;
use crate::{
    grammar::Rule,
    node::{Node, RcNode},
};
use stdx::prelude::*;

/// Bridges the parser with our specific syntax tree representation.
pub(crate) struct TreeBuilder {
    state: State,
    result: Option<RcNode>,
    current: Option<RcNode>,
}

#[derive(Eq, PartialEq)]
enum State {
    PendingStart,
    Normal,
    PendingFinish,
}

impl TreeBuilder {
    fn token(&mut self, token: Token, range: TextRange) {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingStart => unreachable!(),
            State::PendingFinish => self.finish_node(),
            State::Normal => (),
        }
        self.do_token(token, range);
    }

    fn start_node(&mut self, token: Token, range: TextRange) {
        debug!("BuildTree: Starting node: {:?}", token);
        if self.result.is_none() {
            self.result = Some(Node::new(token, range));
            self.current = self.result.clone();
            return;
        }

        let new_node = Node::child_of(self.current.as_mut().unwrap(), token, range);
        self.current = Some(new_node);
        self.state = State::Normal;
    }

    fn finish_node(&mut self) {
        debug!("BuildTree: finishing node");
        match mem::replace(&mut self.state, State::PendingFinish) {
            State::PendingStart => unreachable!(),
            State::PendingFinish => {
                self.current = self.current.as_ref().unwrap().parent.clone();
            }
            State::Normal => (),
        }
    }

    fn error(&mut self, error: String, range: TextRange) {
        debug!("BuildTree: error {:?}", error);
        unsafe {
            Rc::get_mut_unchecked(self.current.as_mut().unwrap())
                .children
                .push(Node::new_error(range, error));
        }
    }

    pub(super) fn new() -> Self {
        Self {
            state: State::PendingStart,
            result: None,
            current: None,
        }
    }

    pub(super) fn finish(mut self) -> RcNode {
        match mem::replace(&mut self.state, State::Normal) {
            State::PendingFinish => self.finish_node(),
            State::PendingStart | State::Normal => unreachable!(),
        }
        self.result.clone().unwrap()
    }

    fn do_token(&mut self, token: Token, range: TextRange) {
        debug!("BuildTree: doing token: {:?}", token);
        unsafe {
            Rc::get_mut_unchecked(self.current.as_mut().unwrap())
                .children
                .push(Node::new(token, range));
        }
    }

    pub fn build(mut events: Vec<ParseEvent>) -> RcNode {
        let mut sink = Self::new();
        let mut forward_parents = Vec::new();
        for i in 0..events.len() {
            match mem::replace(&mut events[i], ParseEvent::tombstone()) {
                ParseEvent::Start {
                    kind: Token::Tombstone,
                    ..
                } => {
                    debug!("BuildTree: Tombstone, skipping");
                }

                ParseEvent::Start {
                    kind,
                    range,
                    forward_parent,
                } => {
                    assert!(kind != Token::Tombstone);
                    debug!("BuildTree: Start({:?})", kind);
                    // For events[A, B, C], B is A's forward_parent, C is B's forward_parent,
                    // in the normal control flow, the parent-child relation: `A -> B -> C`,
                    // while with the magic forward_parent, it writes: `C <- B <- A`.

                    // append `A` into parents.
                    forward_parents.push((kind, range));
                    let mut idx = i;
                    let mut fp = forward_parent;
                    while let Some(fwd) = fp {
                        idx += fwd as usize;
                        // append `A`'s forward_parent `B`
                        fp = match mem::replace(&mut events[idx], ParseEvent::tombstone()) {
                            ParseEvent::Start {
                                kind,
                                range,
                                forward_parent,
                            } => {
                                if kind != Token::Tombstone {
                                    forward_parents.push((kind, range));
                                }
                                forward_parent
                            }
                            _ => unreachable!(),
                        };
                        // append `B`'s forward_parent `C` in the next stage.
                    }

                    for (kind, range) in forward_parents.drain(..).rev() {
                        sink.start_node(kind, range);
                    }
                }
                ParseEvent::Finish => sink.finish_node(),
                ParseEvent::Token(token, range) => {
                    sink.token(token, range);
                }
                ParseEvent::Error(e, range) => sink.error(e, range),
            }
        }
        sink.finish()
    }
}
