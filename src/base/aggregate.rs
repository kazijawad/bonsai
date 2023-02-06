use crate::base::primitive::Primitive;

pub trait Aggregate: Primitive + Send + Sync {}
