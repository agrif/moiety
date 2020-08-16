pub trait ResourceType: Copy {
    type Data;
    fn name(&self) -> &str;
}
