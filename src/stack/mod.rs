use std::{cell::RefCell, rc::Rc};

pub mod api;
pub mod iam;
pub mod lambda;
pub mod region;
pub mod tf;

pub type Shared<T> = Rc<RefCell<T>>;

pub fn shared<T>(t: T) -> Shared<T> {
    Rc::new(RefCell::new(t))
}
