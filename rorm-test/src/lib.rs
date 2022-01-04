use std::future::Future;

use rorm::{Entity, Repository};

pub async fn run_test<E, Fn, Fut>(f: Fn)
where
    E: Entity,
    Fn: FnOnce(Repository<E>) -> Fut,
    Fut: Future<Output = ()>,
{
    env_logger::try_init().ok();

    let conn = rorm::pool::sqlite::Builder::memory().connect().unwrap();
    let repo = Repository::<E>::new(conn);

    repo.init().await.unwrap();

    f(repo).await;
}
