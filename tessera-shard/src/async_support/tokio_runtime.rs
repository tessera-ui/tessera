use std::ops::Deref;

pub(crate) struct TaskExecutor(Option<tokio::runtime::Runtime>);

impl Default for TaskExecutor {
    fn default() -> Self {
        Self(Some(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build shard task executor"),
        ))
    }
}

impl Deref for TaskExecutor {
    type Target = tokio::runtime::Runtime;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("live task executor")
    }
}

impl Drop for TaskExecutor {
    fn drop(&mut self) {
        if let Some(runtime) = self.0.take() {
            runtime.shutdown_background();
        }
    }
}
