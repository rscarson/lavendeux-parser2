use crate::{pest, Error, Node};
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

// A cache of pre-compiled user function bodies
thread_local! {
    static USER_FUNCTION_CACHE: OnceCell<RefCell<HashMap<String, Rc<Node>>>> = OnceCell::new();
}

pub fn cached_fn_compile(src: &'i str, line_offset: usize) -> Result<Rc<Node<'i>>, Error<'i>> {
    USER_FUNCTION_CACHE.with(|once_lock| {
        let rt_mut = once_lock.get_or_init(|| RefCell::new(HashMap::new()));
        let mut cache = rt_mut.borrow_mut();

        match cache.entry(src.to_string()) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => {
                let mut node = pest::parse_input(src, pest::Rule::EXPR)?;
                node.token_offsetline(line_offset);
                Ok(v.insert(Rc::new(node)).clone())
            }
        }
    })
}
