#[derive(Debug, Clone)]
pub enum Expression {}

pub struct ContractDefinition {
    /// Id of the next variable in the sym table.
    pub next_var_id: usize,
}
