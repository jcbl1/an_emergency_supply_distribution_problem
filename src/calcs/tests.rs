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

#[test]
fn test_time_cost() {
    let solution = Solution::random_new();
    let k = 1usize;
    let u = Stage::R;
    let destination = 6usize;
    let route = solution.get_route_of_k_in_stage_u(k, &u);
    let time_cost = solution.time_cost_for_k_to_reach_i_in_stage_u(k, destination, &u);
    println!(
        "Route of vehicle {} in stage {:?}: {:?}, time_cost to reach {}: {}",
        k, u, route, destination, time_cost
    );
}
