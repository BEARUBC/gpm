use crate::ManagerChannelMap;

pub async fn run_gpio_dispatcher_loop(_manager_channel_map: ManagerChannelMap) {
    panic!("Cannot use GPIO monitor outside the Pi");
}
