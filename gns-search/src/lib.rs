//! GNS Search — Spatial-identity discovery
//! Stub crate for compilation. Full implementation pending.

pub struct QueryBuilder;

pub fn query() -> QueryBuilder {
    QueryBuilder
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Gt, Lt, Gte, Lte, Eq, Neq,
}

#[derive(Debug, Clone, Copy)]
pub enum RankingKey {
    Trust, Distance, Recency, Relevance,
}

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Asc, Desc,
}

impl QueryBuilder {
    pub fn filter(self, _field: &str, _op: Op, _val: impl std::fmt::Display) -> Self { self }
    pub fn rank_by(self, _key: RankingKey, _order: Order) -> Self { self }
    pub fn execute(self) -> Vec<()> { vec![] }
}
