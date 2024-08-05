#[macro_export]
macro_rules! verify_task_data {
    ($rcvd:expr, $expected:path, $component:expr) => {
        match $rcvd {
            Some(data) => match data {
                $expected(data) => Some(data),
                _ => {
                    warn!(
                        "Mismatched task data type for task; Expected={:?} Recieved={:?}",
                        $component, data
                    );
                    None
                },
            },
            _ => None,
        };
    };
}

#[macro_export]
macro_rules! _dispatch_task {
    {
        $request:ident,
        $map:ident,
        $(
            $variant:pat
        ),*
    } => {{
        let component_key = $request.component().as_str_name();
        match $request.component() {
            $($variant => {
                info!("Dispatching {:?} task with task_code={:?}", component_key, $request.task_code);
                match $map.get($request.component().as_str_name()) {
                    Some(tx) => {
                        // Set up channel on which manager will send its response
                        let (resp_tx, resp_rx) = oneshot::channel::<String>();
                        tx.send(ManagerChannelData {
                            task_code: $request.task_code.as_str().to_string(),
                            task_data: $request.task_data, 
                            resp_tx 
                        }).await.unwrap();
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
    }}
}
