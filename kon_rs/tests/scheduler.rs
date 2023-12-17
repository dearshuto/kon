use std::collections::{HashMap, HashSet};

use kon_rs::{
    algorithm::{AssignParams, RoomMatrix, Scheduler},
    BandId,
};

#[test]
fn simple() {
    let scheduler = Scheduler::new();

    let condition = AssignParams {
        band_schedule: HashMap::from([(BandId::new(), HashSet::from([]))]),
        room_matrix: RoomMatrix::new(&[1]),
    };
    let _schedule = scheduler.assign(&condition).unwrap();

    panic!();
}
