#[macro_export]
macro_rules! _init_resource_managers {
    {
        $(
            $component:expr => $variant:expr
        ),*
    } => {
        // This map will hold the mappings between task managers and their tx component of their mpsc
        // channels
        let mut map = HashMap::new();
        $(
            let mut manager = $variant;
            info!("Initializing resource_manager_task={:?}", $component.as_str_name());
            manager.init().unwrap();
            map.insert($component.as_str_name().to_string(), manager.tx());
            tokio::spawn(async move { manager.run().await; });
        )*
        map
    };
}

#[macro_export]
macro_rules! match_managers {
    ($id:ident, $method:ident) => {
        match $id {
            Manager::BmsManager(bms) => bms.$method(),
            Manager::EmgManager(emg) => emg.$method(),
            Manager::MaestroManager(maestro) => maestro.$method(),
        }
    };
    ($id:ident, $method:ident, $($arg:ident),*) => {
        match $id {
            Manager::BmsManager(bms) => bms.$method($($arg),*),
            Manager::EmgManager(emg) => emg.$method($($arg),*),
            Manager::MaestroManager(maestro) => maestro.$method($($arg),*),
        }
    }
}

#[macro_export]
macro_rules! async_match_managers {
    ($id:ident, $method:ident) => {
        match $id {
            Manager::BmsManager(bms) => bms.$method().await,
            Manager::EmgManager(emg) => emg.$method().await,
            Manager::MaestroManager(maestro) => maestro.$method().await,
        }
    };
    ($id:ident, $method:ident, $($arg:ident),*) => {
        match $id {
            Manager::BmsManager(bms) => bms.$method($($arg),*).await,
            Manager::EmgManager(emg) => emg.$method($($arg),*).await,
            Manager::MaestroManager(maestro) => maestro.$method($($arg),*).await,
        }
    }
}

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

#[macro_export]
macro_rules! verify_channel_data {
    ($data:ident, $channel_data:path) => {
        match $data {
            $channel_data($data) => {
                info!("Recieved task={:?}", $data.0);
                Ok($data)
            },
            _ => Err(Error::msg(
                "Mismatched channel data type; Expected={$channel_data} Received={$data}",
            )),
        }
    };
}
