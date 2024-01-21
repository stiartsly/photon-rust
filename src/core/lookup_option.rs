#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LookupOption {
    LOCAL,
    ARBITRARY,
    OPTIMISTIC,
    CONSERVATIVE,
}
