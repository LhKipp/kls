use crate::lexer::Token;

use super::Rule;

pub(crate) struct TopLevelStatement {}

impl Rule for TopLevelStatement {
    fn name(&self) -> String {
        "TopLevelStatement".into()
    }

    fn matches(&self, s: &crate::parser::Parser) -> bool {
        s.tokens
            .next_non_ws()
            .is_some_and(|(t, _)| *t == Token::PackageKeyword)
    }

    fn parse_rule(&self, p: &mut crate::parser::Parser) {
        let mut package_decl = p.start(Token::PackageDecl, None);
        p.eat(Token::Ws);
        p.expect(Token::PackageKeyword);
        p.eat(Token::Ws);
        loop {
            p.expect(Token::SimpleIdent);
            if !p.eat(Token::Period) {
                break;
            }
        }
        package_decl.finish(p);
    }
}
