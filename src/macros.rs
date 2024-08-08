/// A few handy macros used across the codebase

/// Imports the protobuf generated code to enable de/serialization
/// TODO: @krarpit this should really be a proc macro that reads from the
///                sgcp folder
#[macro_export]
macro_rules! import_sgcp {
    () => {
        pub mod sgcp {
            include!(concat!(env!("OUT_DIR"), "/sgcp.rs"));
            pub mod bms {
                include!(concat!(env!("OUT_DIR"), "/sgcp.bms.rs"));
            }
            pub mod emg {
                include!(concat!(env!("OUT_DIR"), "/sgcp.emg.rs"));
            }
            pub mod maestro {
                include!(concat!(env!("OUT_DIR"), "/sgcp.maestro.rs"));
            }
        }
        use sgcp::*;
    };
}

/// Simple wrapper to allow retrying on failures
#[macro_export]
macro_rules! retry {
    ($f:expr, $count:expr, $interval:expr) => {{
        let mut retries = 0;
        let result = loop {
            let result = $f;
            if result.is_ok() {
                break result;
            } else if retries > $count {
                break result;
            } else {
                retries += 1;
                tokio::time::sleep(std::time::Duration::from_millis($interval)).await;
            }
        };
        result
    }};
    ($f:expr) => {
        retry!($f, 5, 100)
    };
}

/// todo! without panicking
#[macro_export]
macro_rules! todo {
    () => {
        error!("Not yet implemented")
    };
}

/// Certain methods are only run when GPM is in the Raspberry Pi environment (for eg. GPIO access).
/// This macro must be used to log whenever some task is skipped when running GPM outside
/// the Pi.
#[macro_export]
macro_rules! not_on_pi {
    () => {
        warn!("Not running on the Raspberry Pi -- skipping task")
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

/// Provides boilerplate to parse task information needed by resource managers to
/// proccess tasks
#[macro_export]
macro_rules! parse_channel_data {
    ($id:ident, $task:ty, $task_data:ty) => {{
        let data = verify_channel_data!($id, $task, $task_data).map_err(|e: Error| e)?;
        Ok((data.0, data.1, $id.resp_tx))
    }};
}

/// Generic implementation for implementing the `run` method of a ResourceManager
#[macro_export]
macro_rules! run {
    ($resource:ty) => {
        async fn run(&mut self) {
            info!(
                "{:?} resource manager now listening for messages",
                <$resource>::name()
            );
            while let Some(data) = self.rx.recv().await {
                match self.handle_task(data).await {
                    Err(err) => error!(
                        "Handling {:?} task failed with error={:?}",
                        <$resource>::name(),
                        err
                    ),
                    _ => (),
                };
            }
        }
    };
}
