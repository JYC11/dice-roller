#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Eq(u32),
    Gt(u32),
    Gte(u32),
    Lt(u32),
    Lte(u32),
}
