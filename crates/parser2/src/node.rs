use crate::lexer::Token;
use crate::parser::ParserState;
use crop::Rope;
use ptree::{write_tree, Style, TreeBuilder, TreeItem};
use std::borrow::Cow;
use std::fmt::Pointer;
use std::io::BufWriter;
use std::{fmt::Display, rc::Rc};
use stdx::prelude::*;
use stdx::TextRange;
use strum_macros::IntoStaticStr;

pub type RcNode = Rc<Node>;

pub struct Node {
    pub ntype: Token,
    pub parent: Option<RcNode>,
    pub children: Vec<RcNode>,
    pub range: TextRange,
    // Only set if token is error
    pub err: Option<String>,
}

impl Node {
    pub fn new(ntype: Token, range: TextRange) -> RcNode {
        Rc::new(Self {
            ntype,
            parent: None,
            children: vec![],
            range,
            err: None,
        })
    }

    pub fn new_error(range: TextRange, error: String) -> RcNode {
        Rc::new(Self {
            ntype: Token::Error,
            parent: None,
            children: vec![],
            range,
            err: Some(error),
        })
    }

    pub fn child_of(parent: &mut RcNode, ntype: Token, range: TextRange) -> RcNode {
        let self_ = Rc::new(Self {
            ntype,
            parent: Some(parent.clone()),
            children: vec![],
            range,
            err: None,
        });
        unsafe {
            Rc::get_mut_unchecked(parent).children.push(self_.clone());
        }
        self_
    }

    pub fn text(&self, source: &Rope) -> String {
        return source.byte_slice(self.range.into_usize_range()).to_string();
    }

    pub fn sexp(&self) -> String {
        let mut builder = TreeBuilder::new(format!("{}", self));
        for child in &self.children {
            rc_node_sexp_helper(&*child, &mut builder);
        }

        let mut buf = BufWriter::new(Vec::new());
        write_tree(&builder.build(), &mut buf);

        let bytes = buf.into_inner().unwrap();
        String::from_utf8(bytes).unwrap()
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ntype == Token::Error {
            write!(
                f,
                "{} {:?} {}",
                self.range,
                self.ntype,
                self.err.as_ref().unwrap()
            )?;
        } else {
            write!(f, "{} {:?}", self.range, self.ntype)?;
        }
        Ok(())
    }
}

fn rc_node_sexp_helper(node: &Node, builder: &mut TreeBuilder) {
    if node.children.is_empty() {
        builder.add_empty_child(format!("{}", node));
    } else {
        builder.begin_child(format!("{}", node));
        for child in &node.children {
            rc_node_sexp_helper(&*child, builder);
        }
        builder.end_child();
    }
}
