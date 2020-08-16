pub trait Stack: Sized + std::cmp::Eq + std::hash::Hash {
    fn name(&self) -> &str;
    fn all() -> Vec<Self>;
}
