use crate::invoke;
use crate::game::Handle;

pub unsafe fn set_vehicle_population_budget(budget: u32) {
    invoke!((), 0xCB9E1EB3BE2AF4E9, budget)
}

pub unsafe fn set_ped_population_budget(budget: u32) {
    invoke!((), 0x8C95333CFC3340F3, budget)
}