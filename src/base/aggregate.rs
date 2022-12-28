use crate::base::primitive::Primitive;

pub trait Aggregate<'a>: Primitive<'a> + Send + Sync {}
