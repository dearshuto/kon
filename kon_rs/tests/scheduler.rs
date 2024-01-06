use std::collections::HashMap;

use kon_rs::algorithm::Scheduler;

#[test]
fn simple() {
    let live_info = kon_rs::algorithm::create_live_info(&HashMap::new(), &HashMap::default());
    let scheduler = Scheduler::new();
    let _schedule = scheduler.assign(&[1], &live_info);
}
