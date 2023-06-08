use std::panic::{RefUnwindSafe, UnwindSafe};

// TODO: probably don't derive so we don't expose private fields
#[derive(Debug)]
pub struct AbortHandle;

impl AbortHandle {
    pub fn abort(&self) {
        unimplemented!()
    }

    pub fn is_finished(&self) -> bool {
        unimplemented!()
    }
}

impl Drop for AbortHandle {
    fn drop(&mut self) {
        //TODO:
    }
}

impl UnwindSafe for AbortHandle {}
impl RefUnwindSafe for AbortHandle {}
