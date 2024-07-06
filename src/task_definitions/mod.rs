pub mod bms;
pub mod servo;

trait TaskDefinition {
    fn dispatch_task(self, task_code: i32);
}