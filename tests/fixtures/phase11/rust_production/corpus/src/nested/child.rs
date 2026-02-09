pub fn local_helper() {}

pub fn invoke_super() {
    super::top_helper();
}

pub fn invoke_self() {
    self::local_helper();
}

pub fn invoke_crate() {
    crate::util::helper();
}
