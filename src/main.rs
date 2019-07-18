extern crate election_result_process;
use election_result_process::{get_constituency_name, aggregate_result_by_symbols};
use std::env;


fn main() {
    let args:Vec<String> = env::args().collect();
    aggregate_result_by_symbols(&args[1], &args[2]);
}
