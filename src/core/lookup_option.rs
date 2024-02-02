#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LookupOption {
    Local,
    Arbitrary,
    Optimistic,
    Conservative,
}
