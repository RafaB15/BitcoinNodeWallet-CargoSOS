/// It's the trait that allows to create a default value where could have a posible error in the creation
pub trait TryDefault: Sized {
    type Error;

    fn try_default() -> Result<Self, Self::Error>;
}
