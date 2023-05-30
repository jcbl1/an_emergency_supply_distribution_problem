use std::{eprintln, sync::atomic::Ordering, thread, time::Duration};

use super::*;

#[cfg(test)]
mod tests;

pub mod get_routes;
pub use get_routes::*;

// const WEIGHTS: [f64; 5] = [0.2; 5];
// const WEIGHTS: [f64; 5] = [0.1,0.1,0.35,0.1,0.35];

// static T_O: AtomicUsize = AtomicUsize::new(0);
static T_R: AtomicUsize = AtomicUsize::new(0);
static SHOW_PARTS_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static WEIGHTS: [AtomicUsize; 4] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    // AtomicUsize::new(0),
    // AtomicUsize::new(0),
];

fn max(num1: f64, num2: f64) -> f64 {
    if num1 > num2 {
        num1
    } else {
        num2
    }
}
//DONE:测试
fn diff(num1: f64, num2: f64) -> f64 {
    if num1 > num2 {
        num1 - num2
    } else {
        num2 - num1
    }
}

//DONE:通过
fn demand_with_time(t: f64) -> f64 {
    t.powf(1.08) / 1.66
    // println!("demand with time {} of {}, base {}: {}", t,i,DEMANDS[i], result);
    // result
}

#[derive(Debug)]
pub enum Stage {
    O,
    R,
}

pub trait Calcs {
    fn f1(&self) -> f64;
    fn f2(&self) -> f64;
    fn utility_of_i_in_stage_u(&self, i: usize, u: &Stage) -> f64;
    fn delivered_to_i_in_stage_u(&self, i: usize, u: &Stage) -> f64;
    fn uniformalized_f(&mut self) -> f64;
    fn satisfaction_to_restriction_8(&self) -> f64;
    fn demand_of_i_in_stage_u(&self, i: usize, u: &Stage) -> f64;
    // fn time_cost_for_k_to_reach_i_in_stage_u(&self, k: usize, i: usize, u: &Stage) -> f64;
    fn satisfaction_to_restriction_11(&self) -> f64;
    // fn satisfaction_to_restriction_12(&self) -> f64;
    // fn get_route_of_k_in_stage_u(&self, k: usize, u: &Stage) -> Vec<usize>;
    // fn satisfaction_to_route_circuit(&self) -> f64;
    fn update_totr(&mut self);
    fn update_weights(&self);
    // fn punish_zero_starters(&self) -> f64;
    fn update_routes(&mut self);
}

impl Calcs for Solution {
    //DONE: 修改，简化，考虑最短化时长即为最短化两阶段最大时长之和
    fn f1(&self) -> f64 {
        let mut result = 0.;
        result += self.totr.0 + self.totr.1;

        result
    }
    //TODO: 验证f2()的正确性
    fn f2(&self) -> f64 {
        let mut result = 0f64;
        for i in 0..NUM_CITIES {
            result -= self.utility_of_i_in_stage_u(i, &Stage::O);
            result -= self.utility_of_i_in_stage_u(i, &Stage::R);
        }

        MAX_F2.fetch_max(result as isize, Ordering::Relaxed);
        MIN_F2.fetch_min(result as isize, Ordering::Relaxed);

        result
    }
    fn utility_of_i_in_stage_u(&self, i: usize, u: &Stage) -> f64 {
        let xi = self.delivered_to_i_in_stage_u(i, u);
        let mut utility = xi;
        for j in 0..NUM_CITIES {
            utility -= ALPHA[i]
                * (max(self.delivered_to_i_in_stage_u(j, u) - xi, 0f64)
                    / (NUM_CITIES as f64 - 1f64));
            utility -= BETA[i]
                * (max(xi - self.delivered_to_i_in_stage_u(j, u), 0f64)
                    / (NUM_CITIES as f64 - 1f64));
        }
        utility
    }
    //DONE: 验证
    fn delivered_to_i_in_stage_u(&self, i: usize, u: &Stage) -> f64 {
        let mut sum = 0f64;

        for k in 0..NUM_VEHICLES {
            match u {
                Stage::O => {
                    let route = self.routes_o[k].clone();
                    // if route.len() > 1 && route[0] == route[route.len() - 1] {
                    //     route.pop();
                    // }
                    for city in route {
                        if city == i {
                            let addition = self.xiko[k][i] as f64;
                            sum += addition;
                            break;
                        }
                    }
                }
                Stage::R => {
                    let route = self.routes_r[k].clone();
                    // if route.len() > 1 && route[0] == route[route.len() - 1] {
                    //     route.pop();
                    // }
                    for city in route {
                        if city == i {
                            sum += self.xikr[k][i] as f64;
                            break;
                        }
                    }
                }
            }
            // sum += match u {
            //     Stage::O => self.xiko[k][i] as f64,
            //     Stage::R => self.xikr[k][i] as f64,
            // }
        }

        sum
    }

    fn uniformalized_f(&mut self) -> f64 {
        self.update_routes();
        self.update_totr();
        let mut result = 0f64;
        let (max_f1, min_f1, max_f2, min_f2) = (
            // MAX_F1.load(Ordering::Relaxed) as f64,
            // MIN_F1.load(Ordering::Relaxed) as f64,
            MAX_F1,
            MIN_F1,
            MAX_F2.load(Ordering::Relaxed) as f64,
            MIN_F2.load(Ordering::Relaxed) as f64,
        );
        let max_r8 = MAX_R8.load(Ordering::Relaxed) as f64 / 100f64;
        // println!("max_f1: {}, min_f1: {}, max_f2: {}, min_f2: {}", max_f1,min_f1,max_f2,min_f2);
        self.parts[0] = (self.f1() - min_f1) / (max_f1 - min_f1);
        self.parts[1] = (self.f2() - min_f2) / (max_f2 - min_f2);
        self.parts[2] = self.satisfaction_to_restriction_8() / max_r8;
        self.parts[3] = self.satisfaction_to_restriction_11();
        // self.parts[4] = self.satisfaction_to_restriction_12();
        // self.parts[4] = self.satisfaction_to_route_circuit();
        // self.parts[5] = self.punish_zero_starters();
        // dbg!(&self.parts);
        // if SHOW_PARTS_COUNT.load(Ordering::Relaxed)>5000{
        //     dbg!(&self.parts);
        //     SHOW_PARTS_COUNT.store(0, Ordering::Relaxed);
        // } else{
        //     SHOW_PARTS_COUNT.fetch_add(1, Ordering::Relaxed);
        // }
        // thread::sleep(duration::from_secs(1));
        for (i, part) in self.parts.iter().enumerate() {
            let w = WEIGHTS[i].load(Ordering::Relaxed) as f64 / 10000f64;
            result += w * part;
        }
        if result < 0f64 {
            panic!(
                "uniformalized_f is less than 0. parts: {:?}, max_f2:{},min_f2:{}",
                self.parts, max_f2, min_f2
            );
        }

        if result > 1f64 {
            dbg!(&result, &self.parts, &WEIGHTS);
        }

        // self.update_weights();
        result
    }

    //TODO: 测试
    fn satisfaction_to_restriction_8(&self) -> f64 {
        let mut result = 0f64;
        let mut total_demands: Vec<f64> = Vec::from([0.; 9]);
        let mut delivered: Vec<f64> = Vec::from([0.; 9]);
        for i in 0..NUM_CITIES {
            total_demands[i] = self.demand_of_i_in_stage_u(i, &Stage::O)
                + self.demand_of_i_in_stage_u(i, &Stage::R);
            delivered[i] = self.delivered_to_i_in_stage_u(i, &Stage::O)
                + self.delivered_to_i_in_stage_u(i, &Stage::R);
            // let discrepancy = (total_demands[i] - delivered[i]).powi(2);
            let discrepancy = diff(total_demands[i], delivered[i]);
            result += discrepancy;
        }
        MAX_R8.fetch_max((result * 100f64) as usize, Ordering::Relaxed);
        // let max_r8 = MAX_R8.fetch_max((result * 100f64) as usize, Ordering::Relaxed);
        // if max_r8 > (result * 40000f64) as usize {
        //     MAX_R8.store((max_r8 as f64/1.25)as usize, Ordering::Relaxed);
        // }
        result
    }

    //TODO: test
    fn demand_of_i_in_stage_u(&self, i: usize, u: &Stage) -> f64 {
        let mut result = 0f64;
        match u {
            Stage::O => {
                let demand_i = DEMANDS[i] as f64;
                result += demand_i;
                result += demand_i * demand_with_time(self.totr.0);
            }
            Stage::R => {
                let demand_i = DEMANDS[i] as f64;
                result += demand_i * demand_with_time(self.totr.1);
            }
        }

        result
    }

    // fn time_cost_for_k_to_reach_i_in_stage_u(&self, k: usize, i: usize, u: &Stage) -> f64 {
    //     let mut result = 0f64;
    //     match u {
    //         Stage::O => {
    //             let route = self.get_route_of_k_in_stage_u(k, u);
    //             if route.len() > 0 {
    //                 let mut j0 = &route[0];
    //                 for j in route.iter() {
    //                     if j != j0 {
    //                         result += T[*j0][*j];
    //                         j0 = j;
    //                     }
    //                     if *j == i {
    //                         break;
    //                     }
    //                 }

    //                 T_O.fetch_max((result * 100f64) as usize, Ordering::Relaxed);
    //             }
    //         }
    //         Stage::R => {
    //             result += T_O.load(Ordering::Relaxed) as f64 / 100f64;
    //             let route = self.get_route_of_k_in_stage_u(k, u);
    //             if route.len() > 0 {
    //                 let mut j0 = &route[0];
    //                 for j in route.iter() {
    //                     if j != j0 {
    //                         result += T[*j0][*j];
    //                         j0 = j;
    //                     }
    //                     if *j == i {
    //                         break;
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     result

    //     // rand::thread_rng().gen_range::<f64,_>(2f64..20f64)
    // }

    //DONE: 改写
    //TODO:测试
    fn satisfaction_to_restriction_11(&self) -> f64 {
        let mut result = 0f64;
        let mut city_counts_o = Vec::from([0usize; NUM_CITIES]);
        let mut city_counts_r = Vec::from([0usize; NUM_CITIES]);
        // println!("yijko: {:?}", self.yijko);
        for k in 0..NUM_VEHICLES {
            let route = self.routes_o[k].clone();
            if route.len() != 0 {
                // if route[0] == route[route.len() - 1] && route.len() > 1 {
                //     route.pop();
                // }
                // println!("route of vehicle {} o: {:?}",k, route);
                for city in route {
                    city_counts_o[city] += 1;
                }
            }
            let route = self.routes_r[k].clone();
            if route.len() != 0 {
                // if route[0] == route[route.len() - 1] && route.len() > 1 {
                //     route.pop();
                // }
                // println!("route of vehicle {} r: {:?}", k,route);
                for city in route {
                    city_counts_r[city] += 1;
                }
            }
        }
        for i in 0..NUM_CITIES {
            let discrepancy = city_counts_o[i] as f64 - 1f64;
            let discrepancy = max(discrepancy, 0f64); // discrepancy 最大为9
                                                      // println!("{}", discrepancy);
            result += discrepancy;
            let discrepancy = city_counts_r[i] as f64 - 1f64;
            let discrepancy = max(discrepancy, 0f64); // discrepancy 最大为9
                                                      // println!("{}", discrepancy);
            result += discrepancy;
        }

        result = result / 54f64;
        if result > 1f64 {
            panic!("约束11的目标值大于1了, result: {result}");
        }

        // dbg!(&result);
        result
    }

    //TODO: 改写
    // fn satisfaction_to_restriction_12(&self) -> f64 {
    //     let mut result = 0f64;
    //     for h in 0..NUM_CITIES {
    //         let (mut yin_o, mut yout_o) = (0f64, 0f64);
    //         let (mut yin_r, mut yout_r) = (0f64, 0f64);
    //         for k in 0..NUM_VEHICLES {
    //             for i in 0..NUM_CITIES {
    //                 if self.yijko[k][i][h] {
    //                     yin_o += 1f64;
    //                 }
    //                 if self.yijko[k][h][i] {
    //                     yout_o += 1f64;
    //                 }
    //                 if self.yijkr[k][i][h] {
    //                     yin_r += 1f64;
    //                 }
    //                 if self.yijkr[k][h][i] {
    //                     yout_r += 1f64;
    //                 }
    //             }
    //         }
    //         result += (yin_o - yout_o).powi(2) / 70f64 / 18f64;
    //         result += (yin_r - yout_r).powi(2) / 70f64 / 18f64;
    //     }

    //     if result > 1f64 {
    //         panic!("约束12的目标值大于1了, result: {result}");
    //     }

    //     result
    // }

    // fn get_route_of_k_in_stage_u(&self, k: usize, u: &Stage) -> Vec<usize> {
    //     let mut route = Vec::new();
    //     let from_i = |i: usize, route: &mut Vec<usize>| {
    //         let mut i = i;
    //         loop {
    //             let mut found = false;
    //             'loop1: for j in 0..NUM_CITIES {
    //                 match u {
    //                     Stage::O => {
    //                         if self.yijko[k][i][j] && i != j {
    //                             let mut iter = route.iter();
    //                             iter.next();
    //                             while let Some(existed) = iter.next() {
    //                                 if j == *existed {
    //                                     continue 'loop1;
    //                                 }
    //                             }
    //                             route.push(j);
    //                             i = j;
    //                             if j != route[0] {
    //                                 found = true;
    //                             }
    //                             break;
    //                         }
    //                     }
    //                     Stage::R => {
    //                         if self.yijkr[k][i][j] && i != j {
    //                             let mut iter = route.iter();
    //                             iter.next();
    //                             while let Some(existed) = iter.next() {
    //                                 if j == *existed {
    //                                     continue 'loop1;
    //                                 }
    //                             }
    //                             route.push(j);
    //                             i = j;
    //                             if j != route[0] {
    //                                 found = true;
    //                             }
    //                             break;
    //                         }
    //                     }
    //                 }
    //             }
    //             if !found {
    //                 break;
    //             }
    //         }
    //     };
    //     let i = rand::thread_rng().gen_range::<usize, _>(0..NUM_CITIES);
    //     let range = (i..NUM_CITIES).chain(0..i);
    //     'loop0: for i in range {
    //         for j in 1..NUM_CITIES {
    //             match u {
    //                 Stage::O => {
    //                     if self.yijko[k][i][j] && i != j {
    //                         route.push(i);
    //                         route.push(j);
    //                         from_i(j, &mut route);
    //                         break 'loop0;
    //                     }
    //                 }
    //                 Stage::R => {
    //                     if self.yijkr[k][i][j] && i != j {
    //                         route.push(i);
    //                         route.push(j);
    //                         from_i(j, &mut route);
    //                         break 'loop0;
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     route
    // }

    // fn satisfaction_to_route_circuit(&self) -> f64 {
    //     let mut result = 0f64;
    //     for k in 0..NUM_VEHICLES {
    //         let route = self.routes_o[k].clone();
    //         if route.len() > 0 {
    //             if route[0] != route[route.len() - 1] {
    //                 result += 1f64;
    //             }
    //         }
    //         let route = self.routes_r[k].clone();
    //         if route.len() > 0 {
    //             if route[0] != route[route.len() - 1] {
    //                 result += 1f64;
    //             }
    //         }
    //     }
    //     result / 8f64
    // }

    fn update_totr(&mut self) {
        //用于记录两阶段最大时长
        let (mut max_o, mut max_r) = (0., 0.);
        //每辆车有两阶段
        for k in 0..NUM_VEHICLES {
            let (route_o, route_r) = (self.routes_o[k].clone(), self.routes_r[k].clone());
            if route_o.len() != 0 {
                //记录该车在该阶段的时长
                let mut time = 0f64;
                let mut i = 0;
                for j in 1..route_o.len() {
                    let addition = T[route_o[i]][route_o[j]];
                    time += addition;
                    i = j;
                }
                //返回时间
                time += T[route_o[i]][route_o[0]];
                //如果比最大时长大则更新之
                if time > max_o {
                    max_o = time;
                }
            }

            if route_r.len() != 0 {
                //同上
                let mut time = 0f64;
                let mut i = 0;
                for j in 1..route_r.len() {
                    let addition = T[route_r[i]][route_r[j]];
                    time += addition;
                    i = j
                }
                time += T[route_r[i]][route_r[0]];
                if time > max_r {
                    max_r = time;
                }
            }
        }
        self.totr = (max_o, max_r);
    }

    fn update_weights(&self) {
        let sum = self.parts.iter().sum::<f64>();
        for i in 0..self.parts.len() {
            let part = if self.parts[i] > 0. {
                self.parts[i]
            } else {
                0.0001
            };
            let new_weight = part / sum;
            WEIGHTS[i].store((new_weight * 10000f64) as usize, Ordering::Relaxed);
        }
    }

    // fn punish_zero_starters(&self) -> f64 {
    //     let mut result = 0f64;
    //     for k in 0..NUM_VEHICLES {
    //         if self.routes_o[k].len() > 0 && self.routes_o[k][0] == 0 {
    //             result += 1f64;
    //         }
    //         if self.routes_r[k].len() > 0 && self.routes_r[k][0] == 0 {
    //             result += 1f64;
    //         }
    //     }

    //     result / 8f64
    // }

    fn update_routes(&mut self) {
        let routes = takes_24_u8s_and_returns_8_routes(self.u8s.clone());
        let routes = match routes {
            Ok(rs) => rs,
            Err(e) => {
                panic!("{e}");
            }
        };
        self.routes_o = Vec::from(&routes[0..4]);
        self.routes_r = Vec::from(&routes[4..]);
        // for k in 0..NUM_VEHICLES {
        //     self.routes_o[k] = self.get_route_of_k_in_stage_u(k, &Stage::O);
        //     self.routes_r[k] = self.get_route_of_k_in_stage_u(k, &Stage::R);
        // }
    }
}

impl Solution {
    fn random_new() -> Solution {
        let mut genome: Genome = Vec::new();
        for _ in 0..NUM_VEHICLES {
            for _ in 0..NUM_CITIES {
                let val: u8 = rand::thread_rng().gen_range(0..0b1111_1111);
                genome.push(val);
            }
        }
        for _ in 0..NUM_VEHICLES {
            for _ in 0..NUM_CITIES {
                let val: u8 = rand::thread_rng().gen_range(0..0b1111_1111);
                genome.push(val);
            }
        }
        for _ in 0..24 {
            let val: u8 = rand::thread_rng().gen_range(0..0b1111_1111);
            genome.push(val);
        }

        genome.as_solution()
    }
}
