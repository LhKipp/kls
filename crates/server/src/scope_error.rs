pub type ScopeErrors = Vec<ScopeError>;

#[derive(new)]
pub struct ScopeError {
    pub msg: String,
}
