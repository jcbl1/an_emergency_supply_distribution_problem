use std::assert_eq;

use super::*;

#[test]
fn test_gen_route() {
    let y: [[[bool; 9]; 9]; 4] = [
        [
            [false, false, true, false, false, false, false, false, false],
            [false; 9],
            [false; 9],
            [false, true, false, true, false, false, false, false, false],
            [false, true, false, false, false, true, true, false, false],
            [false, false, false, false, false, true, false, true, false],
            [false, false, false, false, true, false, false, false, true],
            [false, false, false, false, true, false, false, true, false],
            [false, false, false, true, false, false, false, false, false],
        ],
        [
            [false, false, false, false, false, false, false, true, false],
            [false, false, false, false, false, false, false, false, true],
            [false, false, false, false, true, false, false, false, false],
            [false, false, false, false, false, true, false, false, false],
            [true, false, false, false, false, false, false, false, false],
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [false, false, true, false, false, false, false, false, false],
            [true, false, false, false, false, false, false, false, false],
        ],
        [
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [false, false, false, true, false, false, true, true, false],
            [true, true, true, false, false, true, false, false, false],
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [false, false, true, false, false, true, false, true, false],
            [false, true, false, false, false, false, true, true, false],
            [false, false, false, true, false, false, false, false, false],
        ],
        [
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [false, false, true, false, false, false, false, false, false],
            [false, false, false, false, false, true, false, false, false],
            [false, false, false, false, false, false, true, false, false],
            [false, false, false, false, false, true, false, false, false],
            [false, false, false, false, true, false, false, false, false],
            [
                false, false, false, false, false, false, false, false, false,
            ],
            [false, true, false, false, false, false, false, false, true],
        ],
    ];
    let mut solution = Solution::random_new();
    for k in 0..NUM_VEHICLES {
        for i in 0..NUM_CITIES {
            for j in 0..NUM_CITIES {
                solution.yijko[k][i][j] = y[k][i][j];
            }
        }
    }
    let target: Vec<Vec<usize>> = vec![vec![0, 2], vec![0, 7, 2, 4, 0], vec![2, 3, 0], vec![3, 5]];
    for k in 0..NUM_VEHICLES {
        let result = solution.get_route_of_k_in_stage_u(k, &Stage::O);
        assert_eq!(result, target[k]);
    }
    for i in 0..NUM_CITIES {
        println!("{:?}", solution.yijko[0][i]);
    }
    // println!("{:?}", result);
}

// #[test]
// fn test_time_cost() {
//     let solution = Solution::random_new();
//     let k = 1usize;
//     let u = Stage::R;
//     let destination = 6usize;
//     let route = solution.get_route_of_k_in_stage_u(k, &u);
//     let time_cost = solution.time_cost_for_k_to_reach_i_in_stage_u(k, destination, &u);
//     println!(
//         "Route of vehicle {} in stage {:?}: {:?}, time_cost to reach {}: {}",
//         k, u, route, destination, time_cost
//     );
// }

#[test]
fn test_restriction_11() {
    let y: [[[bool; 9]; 9]; 4] = [[
        [false, true, false, false, false, false, false, false, false],
        [false, false, true, false, false, false, false, false, false],
        [false, false, false, true, false, false, false, false, false],
        [false, false, false, false, true, false, false, false, false],
        [false, false, false, false, false, true, false, false, false],
        [false, false, false, false, false, false, true, false, false],
        [false, false, false, false, false, false, false, true, false],
        [false, false, false, false, false, false, false, false, true],
        [true, false, false, false, false, false, false, false, false],
    ]; 4];
    // let y = [[[true; 9]; 9]; 4];
    let mut solution = Solution::random_new();
    for k in 0..NUM_VEHICLES {
        for i in 0..NUM_CITIES {
            for j in 0..NUM_CITIES {
                solution.yijko[k][i][j] = y[k][i][j];
                solution.yijkr[k][i][j] = y[k][i][j];
            }
        }
    }
    let r11 = solution.satisfaction_to_restriction_11();
    println!("{}", r11);
}

#[test]
fn test_f1() {
    let mut solution = Solution::random_new();
    for k in 0..NUM_VEHICLES {
        for i in 0..NUM_CITIES {
            for j in 0..NUM_CITIES {
                solution.yijko[k][i][j] = false;
                solution.yijkr[k][i][j] = false;
            }
        }
    }
    let result = solution.f1();
    println!("{}", result);
}

#[test]
fn test_f2() {
    let solution = Solution::random_new();
    let result = solution.f2();
    println!("{}", result);
}

#[test]
fn test_delivered_to_i_in_stage_u() {
    let solution = Solution::random_new();
    println!("{}", solution.fmt());
    for i in 0..NUM_CITIES {
        let result = solution.delivered_to_i_in_stage_u(i, &Stage::R);
        print!("{},", result);
    }
    println!();
}

#[test]
fn test_utility() {
    let solution = Solution::random_new();
    let u = Stage::O;
    for i in 0..NUM_CITIES {
        // for i in 0..1{
        let result = solution.utility_of_i_in_stage_u(i, &u);
        let xi = solution.delivered_to_i_in_stage_u(i, &u);
        println!("U = {} - sum1/8 - sum2/8 = {}", xi, result);
        // print!("sum1 = ");
        // for j in 0..NUM_CITIES{
        //     print!("max({}-{},0)",solution.delivered_to_i_in_stage_u(j, &u),xi);
        //     if j<NUM_CITIES-1{
        //         print!(" + ")
        //     }
        // }
        // print!("\nsum2 = ");
        // for j in 0..NUM_CITIES{
        //     print!("max({}-{},0)",xi,solution.delivered_to_i_in_stage_u(j, &u));
        //     if j <NUM_CITIES-1{
        //         print!(" + ");
        //     }else{
        //         println!();
        //     }
        // }
    }
}

#[test]
fn find_max_min_of_f2() {
    let (mut max_f2, mut min_f2) = (0f64, 0f64);
    let iter_limit = 1000;
    for _ in 0..iter_limit {
        let solution = Solution::random_new();
        for i in 0..NUM_CITIES {
            let result = solution.f2();
            if result > max_f2 {
                max_f2 = result;
            }
            if result < min_f2 {
                min_f2 = result;
            }
        }
    }
    println!("max: {}, min: {}", max_f2, min_f2);
}

#[test]
fn test_r8() {
    let (mut max, mut min) = (0f64, 0f64);
    let iter_limit = 1000;
    for _ in 0..iter_limit {
        let solution = Solution::random_new();
        let result = solution.satisfaction_to_restriction_8();
        if result > max {
            max = result;
        }
        if result < min {
            min = result;
        }
    }
    println!("max: {}, min: {}", max, min);
}

#[test]
fn test_update_totr() {
    for i in 0..NUM_CITIES {
        for j in 0..NUM_CITIES {
            update_totr(&vec![i, j], &Stage::O);
            let t_o = T_O.load(Ordering::Relaxed) as f64 / 100f64;
            println!("route: {:?},T_O: {}", vec![i, j], t_o);
        }
    }
}

#[test]
fn test_demand() {
    let solution = Solution::random_new();
    let i = 0;
    let u = Stage::O;
    let result = solution.demand_of_i_in_stage_u(i, &u);
    println!("demand of {} in stage {:?}: {}", i, u, result);
    let t_o = T_O.load(Ordering::Relaxed) as f64 / 100f64;
    let t_r = T_R.load(Ordering::Relaxed) as f64 / 100f64;
    println!("T_O: {}, T_R: {}", t_o, t_r);
}
