use stdx::WithTR;

#[derive(Debug, Clone)]
pub struct SFunDecl {
    pub ident: Option<String>,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type_>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub ident: Option<String>,
    pub type_: Option<Type_>,
}

impl Parameter {
    pub fn eq_no_ty(&self, other: &Parameter) -> bool {
        self.ident == other.ident && self.type_ == other.type_
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type_ {
    Unit,
    Simple(String),
}

impl std::fmt::Display for SFunDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "fn")?;
        if let Some(ident) = &self.ident {
            write!(f, " {}", ident)?;
        }
        write!(f, "(")?;
        for parameter in &self.parameters {
            write!(f, "{},", parameter)?;
        }
        write!(f, ")")?;

        if let Some(ret_type) = &self.return_type {
            write!(f, " -> {} ", ret_type)?;
        }

        write!(f, " {{...}}")?;
        Ok(())
    }
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ident) = &self.ident {
            write!(f, "{}", ident)?;
        }
        if let Some(type_) = &self.type_ {
            write!(f, ": {}", type_)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Type_ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type_::Unit => write!(f, "Unit")?,
            Type_::Simple(name) => write!(f, "{}", name)?,
        }
        Ok(())
    }
}
