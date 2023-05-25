use std::{eprintln, sync::atomic::Ordering, thread, time::Duration};

use super::*;

#[cfg(test)]
mod tests;

// const WEIGHTS: [f64; 5] = [0.2; 5];
// const WEIGHTS: [f64; 5] = [0.1,0.1,0.35,0.1,0.35];

static T_O: AtomicUsize = AtomicUsize::new(0);
static SHOW_PARTS_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static mut WEIGHTS: [f64; 6] = [0.1666; 6];

fn max(num1: f64, num2: f64) -> f64 {
    if num1 > num2 {
        num1
    } else {
        num2
    }
}

fn demand_with_time(t: f64) -> f64 {
    t.powf(1.15) / 1.43
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
    fn time_cost_for_k_to_reach_i_in_stage_u(&self, k: usize, i: usize, u: &Stage) -> f64;
    fn satisfaction_to_restriction_11(&self) -> f64;
    fn satisfaction_to_restriction_12(&self) -> f64;
    fn get_route_of_k_in_stage_u(&self, k: usize, u: &Stage) -> Vec<usize>;
    fn satisfaction_to_route_circuit(&self) -> f64;
}

impl Calcs for Solution {
    fn f1(&self) -> f64 {
        let mut result = 0.;
        for k in 0..NUM_VEHICLES {
            for j in 0..NUM_CITIES {
                for i in 0..NUM_CITIES {
                    result += if self.yijko[k][j][i] { T[j][i] } else { 0. };
                    result += if self.yijkr[k][j][i] { T[j][i] } else { 0. };
                }
            }
        }

        MAX_F1.fetch_max(result as isize, Ordering::Relaxed);
        MIN_F1.fetch_min(result as isize, Ordering::Relaxed);

        result
    }
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
            utility -= (ALPHA[i]
                * (max(self.delivered_to_i_in_stage_u(j, u) - xi, 0f64)
                    / (NUM_CITIES as f64 - 1f64)) as f64) as f64;
            utility -= (BETA[i]
                * (xi
                    - max(self.delivered_to_i_in_stage_u(j, u), 0f64) / (NUM_CITIES as f64 - 1f64))
                    as f64) as f64;
        }
        utility
    }
    fn delivered_to_i_in_stage_u(&self, i: usize, u: &Stage) -> f64 {
        let mut sum = 0f64;

        for k in 0..NUM_VEHICLES {
            sum += match u {
                Stage::O => self.xiko[k][i] as f64,
                Stage::R => self.xikr[k][i] as f64,
            }
        }

        sum
    }

    fn uniformalized_f(&mut self) -> f64 {
        let mut result = 0f64;
        let (max_f1, min_f1, max_f2, min_f2) = (
            MAX_F1.load(Ordering::Relaxed) as f64,
            MIN_F1.load(Ordering::Relaxed) as f64,
            MAX_F2.load(Ordering::Relaxed) as f64,
            MIN_F2.load(Ordering::Relaxed) as f64,
        );
        // println!("max_f1: {}, min_f1: {}, max_f2: {}, min_f2: {}", max_f1,min_f1,max_f2,min_f2);
        self.parts[0] = (self.f1() - min_f1) / (max_f1 - min_f1);
        self.parts[1] = (self.f2() - min_f2) / (max_f2 - min_f2);
        self.parts[2] = self.satisfaction_to_restriction_8();
        self.parts[3] = self.satisfaction_to_restriction_11();
        self.parts[4] = self.satisfaction_to_restriction_12();
        self.parts[5] = self.satisfaction_to_route_circuit();
        // dbg!(&parts);
        // if SHOW_PARTS_COUNT.load(Ordering::Relaxed)>5000{
        //     dbg!(&self.parts);
        //     SHOW_PARTS_COUNT.store(0, Ordering::Relaxed);
        // } else{
        //     SHOW_PARTS_COUNT.fetch_add(1, Ordering::Relaxed);
        // }
        // thread::sleep(Duration::from_secs(1));
        for (i, part) in self.parts.iter().enumerate() {
            unsafe {
                result += WEIGHTS[i] * part;
            }
        }
        if result <= 0f64 {
            panic!("uniformalized_f is less than 0. parts: {:?}", self.parts);
        }

        result
    }

    fn satisfaction_to_restriction_8(&self) -> f64 {
        let mut result = 0f64;
        for i in 0..NUM_CITIES {
            let (mut sum1, mut sum2) = (0f64, 0f64);
            for k in 0..NUM_VEHICLES {
                sum1 += self.xiko[k][i] as f64;
                sum2 += self.xikr[k][i] as f64;
                let delinquency = (self.demand_of_i_in_stage_u(i, &Stage::O)
                    + self.demand_of_i_in_stage_u(i, &Stage::R)
                    - sum1
                    - sum2)
                    .powi(2);
                // dbg!(&delinquency);
                result += (delinquency) / (2720200f64.powi(2)) / 9f64;
            }
        }

        // let result_of_r_8 =result;
        // dbg!(&result_of_r_8);
        // thread::sleep(Duration::from_secs(1));
        if result > 1f64 {
            eprintln!("约束8的目标值大于1了, result: {result}");
            eprintln!("Routes:");
            for k in 0..NUM_VEHICLES {
                eprintln!("{:?}", self.get_route_of_k_in_stage_u(k, &Stage::O));
                eprintln!("{:?}", self.get_route_of_k_in_stage_u(k, &Stage::R));
            }
            panic!();
        }
        result
    }

    fn demand_of_i_in_stage_u(&self, i: usize, u: &Stage) -> f64 {
        let mut result = 0f64;
        match u {
            Stage::O => {
                result += DEMANDS[i] as f64;
                'outer: for k in 0..NUM_VEHICLES {
                    for j in 0..NUM_CITIES {
                        if self.yijko[k][j][i] {
                            result += DEMANDS[i] as f64
                                * demand_with_time(
                                    self.time_cost_for_k_to_reach_i_in_stage_u(k, i, u),
                                );
                            break 'outer;
                        }
                    }
                }
            }
            Stage::R => {
                result += self.demand_of_i_in_stage_u(i, &Stage::O);
                let mut count = false;
                'outer: for k in 0..NUM_VEHICLES {
                    for j in 0..NUM_CITIES {
                        if self.yijko[k][j][i] {
                            result -= self.xiko[k][i] as f64;
                            if count {
                                break 'outer;
                            } else {
                                count = true;
                            }
                        }
                        if self.yijkr[k][j][i] {
                            result += DEMANDS[i] as f64
                                * demand_with_time(
                                    self.time_cost_for_k_to_reach_i_in_stage_u(k, i, u)
                                        - T_O.load(Ordering::Relaxed) as f64 / 100f64,
                                );
                            if count {
                                break 'outer;
                            } else {
                                count = true;
                            }
                        }
                    }
                }
            }
        }

        result
    }

    fn time_cost_for_k_to_reach_i_in_stage_u(&self, k: usize, i: usize, u: &Stage) -> f64 {
        let mut result = 0f64;
        match u {
            Stage::O => {
                let route = self.get_route_of_k_in_stage_u(k, u);
                if route.len() > 0 {
                    let mut j0 = &route[0];
                    for j in route.iter() {
                        if j != j0 {
                            result += T[*j0][*j];
                            j0 = j;
                        }
                        if *j == i {
                            break;
                        }
                    }

                    T_O.fetch_max((result * 100f64) as usize, Ordering::Relaxed);
                }
            }
            Stage::R => {
                result += T_O.load(Ordering::Relaxed) as f64 / 100f64;
                let route = self.get_route_of_k_in_stage_u(k, u);
                if route.len() > 0 {
                    let mut j0 = &route[0];
                    for j in route.iter() {
                        if j != j0 {
                            result += T[*j0][*j];
                            j0 = j;
                        }
                        if *j == i {
                            break;
                        }
                    }
                }
            }
        }

        result

        // rand::thread_rng().gen_range::<f64,_>(2f64..20f64)
    }

    fn satisfaction_to_restriction_11(&self) -> f64 {
        let mut result = 0f64;
        let mut city_counts_o = Vec::from([0usize; NUM_CITIES]);
        let mut city_counts_r = Vec::from([0usize; NUM_CITIES]);
        // println!("yijko: {:?}", self.yijko);
        for k in 0..NUM_VEHICLES {
            let mut route = self.get_route_of_k_in_stage_u(k, &Stage::O);
            if route[0] == route[route.len() - 1] && route.len() > 1 {
                route.pop();
            }
            // println!("route of vehicle {} o: {:?}",k, route);
            for city in route {
                city_counts_o[city] += 1;
            }
            let mut route = self.get_route_of_k_in_stage_u(k, &Stage::R);
            if route[0] == route[route.len() - 1] && route.len() > 1 {
                route.pop();
            }
            // println!("route of vehicle {} r: {:?}", k,route);
            for city in route {
                city_counts_r[city] += 1;
            }
        }
        // println!("{:?}", city_counts_o);
        // println!("{:?}", city_counts_r);
        for i in 0..NUM_CITIES {
            let discrepancy = city_counts_o[i] as f64 - 1f64;
            let discrepancy = max(discrepancy as f64, 0f64).powi(2); // discrepancy 最大为9
                                                                     // println!("{}", discrepancy);
            result += discrepancy;
            let discrepancy = city_counts_r[i] as f64 - 1f64;
            let discrepancy = max(discrepancy as f64, 0f64).powi(2); // discrepancy 最大为9
                                                                     // println!("{}", discrepancy);
            result += discrepancy;
        }
        // for i in 0..NUM_CITIES {
        //     let (mut delinquency_o, mut delinquency_r) = (0f64, 0f64);
        //     for j in 0..NUM_CITIES {
        //         for k in 0..NUM_VEHICLES {
        //             if self.yijko[k][j][i] {
        //                 delinquency_o += 1f64;
        //             }
        //             if self.yijkr[k][j][i] {
        //                 delinquency_r += 1f64;
        //             }
        //         }
        //     }
        //     delinquency_o -= 1f64;
        //     delinquency_r -= 1f64;
        //     let delinquency_o = max(delinquency_o, 0f64).powi(2);
        //     let delinquency_r = max(delinquency_r, 0f64).powi(2);
        //     // dbg!(&delinquency_o,&delinquency_r);
        //     let delinquency_o = delinquency_o / 450f64 / 18f64;
        //     let delinquency_r = delinquency_r / 450f64 / 18f64;
        //     // dbg!("after", &delinquency_o,&delinquency_r);
        //     result += delinquency_o;
        //     result += delinquency_r;
        // }

        result = result / 162f64;
        if result > 1f64 {
            panic!("约束11的目标值大于1了, result: {result}");
        }

        // dbg!(&result);
        result
    }

    fn satisfaction_to_restriction_12(&self) -> f64 {
        let mut result = 0f64;
        for h in 0..NUM_CITIES {
            let (mut yin_o, mut yout_o) = (0f64, 0f64);
            let (mut yin_r, mut yout_r) = (0f64, 0f64);
            for k in 0..NUM_VEHICLES {
                for i in 0..NUM_CITIES {
                    if self.yijko[k][i][h] {
                        yin_o += 1f64;
                    }
                    if self.yijko[k][h][i] {
                        yout_o += 1f64;
                    }
                    if self.yijkr[k][i][h] {
                        yin_r += 1f64;
                    }
                    if self.yijkr[k][h][i] {
                        yout_r += 1f64;
                    }
                }
            }
            result += (yin_o - yout_o).powi(2) / 70f64 / 18f64;
            result += (yin_r - yout_r).powi(2) / 70f64 / 18f64;
        }

        if result > 1f64 {
            panic!("约束12的目标值大于1了, result: {result}");
        }

        result
    }

    fn get_route_of_k_in_stage_u(&self, k: usize, u: &Stage) -> Vec<usize> {
        let mut route = Vec::new();
        let from_i = |i: usize, route: &mut Vec<usize>| {
            let mut i = i;
            loop {
                let mut found = false;
                'loop1: for j in 0..NUM_CITIES {
                    match u {
                        Stage::O => {
                            if self.yijko[k][i][j] && i != j {
                                let mut iter = route.iter();
                                iter.next();
                                while let Some(existed) = iter.next() {
                                    if j == *existed {
                                        continue 'loop1;
                                    }
                                }
                                route.push(j);
                                i = j;
                                if j != route[0] {
                                    found = true;
                                }
                                break;
                            }
                        }
                        Stage::R => {
                            if self.yijkr[k][i][j] && i != j {
                                let mut iter = route.iter();
                                iter.next();
                                while let Some(existed) = iter.next() {
                                    if j == *existed {
                                        continue 'loop1;
                                    }
                                }
                                route.push(j);
                                i = j;
                                if j != route[0] {
                                    found = true;
                                }
                                break;
                            }
                        }
                    }
                }
                if !found {
                    break;
                }
            }
        };
        'loop0: for i in 0..NUM_CITIES {
            for j in 1..NUM_CITIES {
                match u {
                    Stage::O => {
                        if self.yijko[k][i][j] && i != j {
                            route.push(i);
                            route.push(j);
                            from_i(j, &mut route);
                            break 'loop0;
                        }
                    }
                    Stage::R => {
                        if self.yijkr[k][i][j] && i != j {
                            route.push(i);
                            route.push(j);
                            from_i(j, &mut route);
                            break 'loop0;
                        }
                    }
                }
            }
        }

        route
    }

    fn satisfaction_to_route_circuit(&self) -> f64 {
        let mut result = 0f64;
        for k in 0..NUM_VEHICLES {
            let route = self.get_route_of_k_in_stage_u(k, &Stage::O);
            if route[0] != route[route.len() - 1] {
                result += 1f64;
            }
            let route = self.get_route_of_k_in_stage_u(k, &Stage::R);
            if route[0] != route[route.len() - 1] {
                result += 1f64;
            }
        }
        result / 8f64
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

        for _ in 0..NUM_VEHICLES {
            for _ in 0..NUM_CITIES {
                for _ in 0..NUM_CITIES {
                    genome.push(if rand::thread_rng().gen_bool(0.5) {
                        0b1111_1111
                    } else {
                        0
                    });
                }
            }
        }
        for _ in 0..NUM_VEHICLES {
            for _ in 0..NUM_CITIES {
                for _ in 0..NUM_CITIES {
                    genome.push(if rand::thread_rng().gen_bool(0.5) {
                        0b1111_1111
                    } else {
                        0
                    });
                }
            }
        }

        genome.as_solution()
    }
}
