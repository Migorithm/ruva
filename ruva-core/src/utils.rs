use crate::prelude::AtomicContextManager;

pub trait TClone {
	fn clone(&self) -> Self;
}

pub trait TCloneContext {
	fn clone_context(&self) -> AtomicContextManager;
}

