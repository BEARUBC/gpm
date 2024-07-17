// The following macros abstract away the logic needed to initialize resource managers
// and dispatch incoming tasks
#[macro_export]
macro_rules! _dispatch_task {
    {
        $(
            $variant:pat => ($task_type:ty,$task_data:path,$channel_data:path)
        ),*
    } => {
        pub async fn dispatch_task(request: Request, map: &ManagerChannelMap) -> Result<String> {
            let component_key = request.component().as_str_name();
            match request.component() {
                $($variant => {
                    info!("Dispatching {:?} task with task_code={:?}", component_key, request.task_code);
                    match map.get(request.component().as_str_name()) {
                        Some(tx) => {
                            let (resp_tx, resp_rx) = oneshot::channel::<String>();
                            let task = <$task_type>::from_str_name(request.task_code.as_str()).unwrap();
                            let task_data = match request.task_data {
                                Some(data) => match data {
                                    $task_data(data) => Some(data),
                                    _ => None,
                                },
                                _ => None
                            };
                            tx.send($channel_data(((task, task_data), resp_tx))).await.unwrap();
                            let res = resp_rx.await.unwrap();
                            info!("{:?} task returned value={:?}", component_key, res);
                            Ok(res)
                        },
                        None => {
                            error!("{:?} resource manager not initialized", component_key);
                            Ok("Error".to_string())
                        }
                    }
                }),*,
                _ => {
                    error!("Unmatched task!");
                    Ok("".to_string())
                }
            }
        }
    }
}

#[macro_export]
macro_rules! _init_resource_managers {
    {
        $(
            $component:expr => $variant:expr
        ),*
    } => {
        async fn init_resource_managers() -> ManagerChannelMap {
            let mut map = HashMap::new();
            $(
                let mut manager = $variant;
                info!("Initializing resource_manager_task={:?}", $component.as_str_name());
                manager.init().unwrap();
                map.insert($component.as_str_name().to_string(), manager.tx());
                tokio::spawn(async move { manager.run().await; });
            )*
            map
        }
    };
}

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
