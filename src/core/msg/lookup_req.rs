
use crate::id::Id;

pub(crate) struct Data {
    target: Option<Id>,
    want4: bool,
    want6: bool,
    want_token: bool,
}

impl Data {
    pub(crate) fn new() -> Self {
        Self {
            target: None,
            want4: false,
            want6: false,
            want_token: false,
        }
    }
}

pub(crate) trait Msg {
    fn data(&self) -> &Data;
    fn data_mut(&mut self) -> &mut Data;

    fn target(&self) -> &Id {
        &self.data().target.as_ref().unwrap()
    }

    fn want4(&self) -> bool {
        self.data().want4
    }

    fn want6(&self) -> bool {
        self.data().want6
    }

    fn want_token(&self) -> bool {
        self.data().want_token
    }

    fn want(&self) -> i32 {
        let mut want = 0;

        if self.want4() {
            want |= 0x01
        }
        if self.want6() {
            want |= 0x02
        }
        if self.want_token() {
            want |= 0x04
        }
        want
    }

    fn with_target(&mut self, target: Id) {
        self.data_mut().target = Some(target)
    }

    fn with_want4(&mut self, want: bool) {
        self.data_mut().want4 = want
    }

    fn with_want6(&mut self, want: bool) {
        self.data_mut().want6 = want
    }

    fn with_want_token(&mut self) {
        self.data_mut().want_token = true
    }
}
