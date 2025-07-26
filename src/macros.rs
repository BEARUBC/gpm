// A few handy macros used across the codebase

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

/// Collects the values of a *C-like* struct into a vec. Note that this macro
/// assumes the enum derives prost::enumeration (or implements TryFrom<i32>)
#[macro_export]
macro_rules! enum_values {
    ($enum:path) => {{
        std::iter::successors(Some(1), |&i| Some(i + 1))
            .map_while(|i| <$enum as std::convert::TryFrom<i32>>::try_from(i).ok())
            .collect::<Vec<$enum>>()
    }};
}

/// Extract the task code names from the Task enum definitions generated for the
/// resources
#[macro_export]
macro_rules! get_task_names {
    ($enum:path) => {
        enum_values!($enum)
            .iter()
            .map(|task| task.as_str_name().to_owned())
            .collect::<Vec<String>>()
    };
}
