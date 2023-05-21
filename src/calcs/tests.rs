use std::assert_eq;

use super::*;

#[test]
fn test_gen_route(){
    let solution = Solution::random_new();
    let result=solution.get_route_of_k_in_stage_u(0,&Stage::O);
    for i in 0..NUM_CITIES{
        println!("{:?}", solution.yijko[0][i]);
    }
    // println!("{:?}",solution.yijko[0]);
    println!("{:?}",result);
    // dbg!(&solution.yijko[0]);
    // dbg!(&result);
}

#[test]
fn test_time_cost(){
    let solution=Solution::random_new();
    let k = 1usize;
    let u = Stage::R;
    let destination = 6usize;
    let route=solution.get_route_of_k_in_stage_u(k,&u);
    let time_cost=solution.time_cost_for_k_to_reach_i_in_stage_u(k,destination,&u);
    println!("Route of vehicle {} in stage {:?}: {:?}, time_cost to reach {}: {}",k,u, route,destination,time_cost);
}
