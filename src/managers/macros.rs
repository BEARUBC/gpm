/// Provides boilerplate to verify that the correct type of task and task data is
/// received by a resource manager
macro_rules! parse_channel_data {
    ($data:ident, $task_type:path, $task_data:path) => {{
        let task = <$task_type>::from_str_name($data.task_code.as_str())
            .ok_or(Error::msg("Invalid task"))?;
        let task_data = match $data.task_data {
            Some(data) => match data {
                $task_data(data) => Ok(Some(data)),
                _ => Err(Error::msg("Mismatched task data type")),
            },
            None => Ok(None),
        }?;
        Ok((task, task_data, $data.resp_tx))
    }};
}

/// Provides boilerplate to initialize a resource manager and run it in its own (green) thread
macro_rules! init_resource_managers {
    {$($resource:expr => $manager_arc:expr),*} => {{
        use std::sync::Arc;

        let mut map = std::collections::HashMap::new();

        $(
            info!("Initialising {:?} resource manager task", $resource.as_str_name());

            // clone the Arc so we can lock later
            let manager_clone = Arc::clone(&$manager_arc);

            // spawn an async block to extract tx and run the manager
            let tx = {
                let mgr = manager_clone.lock().await; // lock to access tx
                mgr.tx.clone()
            };
            map.insert($resource.as_str_name().to_string(), tx);

            // spawn the manager task
            let manager_clone = Arc::clone(&$manager_arc);
            tokio::spawn(async move {
                let mut manager = manager_clone.lock().await;
                manager.run().await;
            });
        )*

        map
    }};
}

pub(crate) use init_resource_managers;
pub(crate) use parse_channel_data;
