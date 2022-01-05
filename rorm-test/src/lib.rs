#[macro_export]
macro_rules! run_async_test {
    (( $($table:ident : $repo_ty:ty),* ) => $body:block) => {
        env_logger::try_init().ok();

        let conn = rorm::pool::sqlite::Builder::memory().connect().unwrap();

        let func = |$($table: $repo_ty),*| async move { $body };

        func(
            $(
                {
                    let repo = <$repo_ty>::new(conn.clone());
                    repo.init().await.unwrap();

                    repo
                }
            ),*
        ).await;
    };
}
