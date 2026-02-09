mod nested;
mod support;
mod util;

pub fn run() {
    util::helper();
    support::helper();
    nested::child::invoke_super();
    nested::child::invoke_self();
    nested::child::invoke_crate();
}
