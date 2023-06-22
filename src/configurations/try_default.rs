pub trait TryDefault: Sized {
    type Error;

    fn try_default() -> Result<Self, Self::Error>;
}
