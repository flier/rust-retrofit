use std::error::Error;
use std::future::Future;

pub trait Call<T> {
    type Error: Error + Send + Sync;

    fn execute(self) -> Result<T, Self::Error>;
}

pub trait AsyncCall<T>: Future<Output = T> {}

pub trait Service {
    type Error: Error + Send + Sync;
    type Body;
}
