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
                            // Set up channel on which manager will send its response
                            let (resp_tx, resp_rx) = oneshot::channel::<String>();
                            let task = <$task_type>::from_str_name(request.task_code.as_str()).ok_or(Error::msg("Invalid task type"))?;
                            // We should only parse the task data if it is for the correct task type
                            // i.e we should warn if the user sends Emg task data for a Bms task
                            let task_data = match request.task_data {
                                Some(data) => match data {
                                    $task_data(data) => Some(data),
                                    _ => {
                                        warn!("Received task of type={:?} but task data for another task type; data={:?}", component_key, data);
                                        None
                                    }
                                },
                                _ => None
                            };
                            tx.send($channel_data(((task, task_data), resp_tx))).await.unwrap();
                            let res = resp_rx.await.unwrap();
                            info!("{:?} task returned value={:?}", component_key, res);
                            Ok(res)
                        },
                        None => {
                            Err(Error::msg("{component_key} resource manager not initialized"))
                        }
                    }
                }),*,
                _ => {
                    Err(Error::msg("Unmatched task"))
                }
            }
        }
    }
}
