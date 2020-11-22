use std::error::Error;
use std::future::Future;

pub trait Call<T>: AsyncCall<T> {
    fn send(self) -> Result<T, Self::Error>;
}

pub trait AsyncCall<T> {
    type Error: Error + Send + Sync;
    type Future: Future<Output = Result<T, Self::Error>>;

    fn async_send(self) -> Self::Future;
}

pub trait Service {
    type Error: Error + Send + Sync;
    type Body;
    type Form;
}
