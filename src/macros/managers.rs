/// Provides boilerplate to initialize a resource manager and run it in its own (green) thread
#[macro_export]
macro_rules! _init_resource_managers {
    {
        $(
            $resource:expr => $variant:expr
        ),*
    } => {
        let mut map = HashMap::new();
        $(
            info!("Initializing resource_manager_task={:?}", $resource.as_str_name());
            let mut manager = $variant;
            map.insert($resource.as_str_name().to_string(), manager.tx());
            tokio::spawn(async move { manager.run().await; });
        )*
        map
    };
}

/// Provides boilerplate for the main task listener loop for a resource manager
#[macro_export]
macro_rules! run_task {
    ($id:ident, $handler:ident) => {
        info!("Listening for messages");
        while let Some(data) = $id.rx.recv().await {
            match $id.$handler(data) {
                Err(err) => error!("Handling task failed with error={:?}", err),
                _ => (),
            };
        }
    };
}

/// Provides boilerplate to verify that the correct type of task and task data is
/// received by a resource manager
#[macro_export]
macro_rules! verify_channel_data {
    ($data:ident, $task_type:path, $task_data:path) => {{
        let task = <$task_type>::from_str_name($data.task_code.as_str())
            .ok_or(Error::msg("Invalid task"))?;
        let task_data = match $data.task_data {
            Some(data) => match data {
                $task_data(data) => Ok(Some(data)),
                _ => Err(Error::msg("Mismatch task data type")),
            },
            None => Ok(None),
        }?;
        Ok((task, task_data))
    }};
}
