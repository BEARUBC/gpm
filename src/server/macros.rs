/// Provides the boilerplate to setup routing required to send tasks to the appropriate
/// resource manager
macro_rules! dispatch_task {
    {$request:ident, $($variant:pat => $channel:expr),*} => {{
        let resource_key = $request.resource().as_str_name();

        match $request.resource() {
            $($variant => {
                info!("Dispatching {:?} task with task_code={:?}", resource_key, $request.task_code);

                match $channel {
                    Some(tx) => {
                        // Set up channel on which manager will send its response
                        let (resp_tx, resp_rx) = oneshot::channel::<String>();

                        tx.send(ManagerChannelData {
                            task_code: $request.task_code.as_str().to_string(),
                            task_data: $request.task_data,
                            resp_tx
                        })
                        .await
                        .map_err(|e| Error::msg(format!("Failed to send command to {} manager: {:?}", resource_key, e)))?;

                        let res = resp_rx
                            .await
                            .map_err(|e| Error::msg(format!("Failed to read response from {} manager: {:?}", resource_key, e)))?;

                        info!("{} task returned value={:?}", resource_key, res);

                        Ok(res)
                    },
                    None => {
                        Err(Error::msg(format!("{} resource manager not initialized", resource_key)))
                    }
                }
            }),*,
            _ => {
                Err(Error::msg("Unmatched task"))
            }
        }
    }}
}

pub(crate) use dispatch_task;
