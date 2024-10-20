use std::ops::Index;
use std::ops::Range;
use std::ops::RangeFrom;
use strum_macros::IntoStaticStr;

use logos::Logos;
use stdx::TextRange;

#[derive(Logos, Debug, Eq, PartialEq, Clone, Copy, IntoStaticStr)]
#[logos()] // Ignore this regex pattern between tokens
pub enum Token {
    #[regex(r"[ \t\n\f]+")]
    Ws,
    #[token(".")]
    Period,
    // Or regular expressions.
    #[regex("[a-zA-Z_]+[a-zA-Z_0-9]*")]
    SimpleIdent,

    #[token("package")]
    PackageKeyword,

    // Node types
    Error,
    Tombstone,
    SourceFile,
    PackageDecl,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub(crate) fn lex_string(text: &str, offset: u32) -> TokenVec {
    let tokens = Token::lexer(text)
        .spanned()
        .map(|(t, span)| {
            (
                t.unwrap(),
                TextRange::try_from(span).unwrap().shift_right_by(offset),
            )
        })
        .collect();
    TokenVec {
        tokens,
        cur_elem: 0,
    }
}

pub(crate) struct TokenVec {
    pub tokens: Vec<(Token, TextRange)>,
    pub cur_elem: u32,
}

impl Index<usize> for TokenVec {
    type Output = (Token, TextRange);

    fn index(&self, n: usize) -> &Self::Output {
        &self.tokens[(self.cur_elem as usize + n)]
    }
}

impl Index<Range<usize>> for TokenVec {
    type Output = [(Token, TextRange)];

    fn index(&self, n: Range<usize>) -> &Self::Output {
        let r = Range {
            start: self.cur_elem as usize + n.start,
            end: self.cur_elem as usize + n.end,
        };
        &self.tokens[r]
    }
}

impl Index<RangeFrom<usize>> for TokenVec {
    type Output = [(Token, TextRange)];

    fn index(&self, n: RangeFrom<usize>) -> &Self::Output {
        let r = RangeFrom {
            start: self.cur_elem as usize + n.start,
        };
        &self.tokens[r]
    }
}

impl TokenVec {
    pub fn next_non_ws(&self) -> Option<&(Token, TextRange)> {
        self.tokens[self.cur_elem as usize..]
            .iter()
            .find(|t| t.0 != Token::Ws)
    }
    pub fn current(&self) -> Option<&(Token, TextRange)> {
        self.tokens.get(self.cur_elem as usize)
    }
    pub fn currently_at_as_range(&self) -> Option<TextRange> {
        self.tokens
            .get(self.cur_elem as usize)
            .map(|(_, r)| TextRange::new(r.start, r.start))
    }

    pub(crate) fn bump(&mut self) {
        self.cur_elem += 1;
    }
}
