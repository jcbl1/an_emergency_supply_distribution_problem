use super::*;
use rand::Rng;

// #[test]
// fn test_decode() {
//     let mut genome: Genome = Vec::new();
//     for _ in 0..NUM_VEHICLES {
//         for _ in 0..NUM_CITIES {
//             let val: u8 = rand::thread_rng().gen_range(0..0b1111_1111);
//             genome.push(val);
//         }
//     }
//     for _ in 0..NUM_VEHICLES {
//         for _ in 0..NUM_CITIES {
//             let val: u8 = rand::thread_rng().gen_range(0..0b1111_1111);
//             genome.push(val);
//         }
//     }

//     for _ in 0..NUM_VEHICLES {
//         for _ in 0..NUM_CITIES {
//             for _ in 0..NUM_CITIES {
//                 genome.push(if rand::thread_rng().gen_bool(0.5) {
//                     0b1111_1111
//                 } else {
//                     0
//                 });
//             }
//         }
//     }
//     for _ in 0..NUM_VEHICLES {
//         for _ in 0..NUM_CITIES {
//             for _ in 0..NUM_CITIES {
//                 genome.push(if rand::thread_rng().gen_bool(0.5) {
//                     0b1111_1111
//                 } else {
//                     0
//                 });
//             }
//         }
//     }
//     assert_eq!(genome.len(), TOTAL_LEN);

//     let solution = genome.as_solution();

//     assert!(solution.xiko.len() == NUM_VEHICLES && solution.xiko[0].len() == NUM_CITIES);
//     assert!(solution.yijkr.len() == NUM_VEHICLES);
//     assert!(solution.yijkr[0].len() == NUM_CITIES);
//     assert_eq!(solution.yijkr[0][0].len(), NUM_CITIES);
//     assert!(solution.yijkr[0][0].len() == NUM_CITIES);

//     assert_eq!(solution.yijko[0][0][0], true);
// }

// #[test]
// fn max_f1() {
//     let xiko: Vec<Vec<usize>> = Vec::new();
//     let xikr: Vec<Vec<usize>> = Vec::new();
//     let mut yijko: Vec<Vec<Vec<bool>>> = Vec::new();
//     let mut yijkr: Vec<Vec<Vec<bool>>> = Vec::new();
//     for k in 0..NUM_VEHICLES {
//         yijko.push(Vec::new());
//         yijkr.push(Vec::new());
//         for i in 0..NUM_CITIES {
//             yijko[k].push(Vec::new());
//             yijkr[k].push(Vec::new());
//             for _ in 0..NUM_CITIES {
//                 yijko[k][i].push(true);
//                 yijkr[k][i].push(true);
//             }
//         }
//     }
//     let sol = Solution {
//         xiko,
//         xikr,
//         yijko,
//         yijkr,
//         parts: Vec::from([0.; 5]),
//         totr: (0., 0.),
//         routes_o: vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()],
//         routes_r: vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()],
//     };
//     let max = sol.f1();
//     assert_eq!(max, 123f64);
// }

#[test]
fn max_f2() {
    let N = [
        37209, 34583, 33075, 32145, 26916, 15453, 13476, 10560, 10006,
    ];
    let mut sum = 0i32;
    for elem in N.iter() {
        sum += elem;
    }
    let avg = sum / 9;
    assert_eq!(avg, 18);
}
