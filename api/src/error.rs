use anyhow::anyhow;

pub struct AppError(pub anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl AppError {
    pub fn from_str<E>(err: E) -> Self
    where
        E: ToString,
    {
        AppError(anyhow!(err.to_string()))
    }
}
