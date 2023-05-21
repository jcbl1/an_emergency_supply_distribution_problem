use std::{dbg, println, sync::atomic::{AtomicUsize, AtomicIsize}};

use genevo::{
    operator::prelude::RandomValueMutator, population::ValueEncodedGenomeBuilder, prelude::*,
    recombination::discrete::MultiPointCrossBreeder, reinsertion::elitist::ElitistReinserter,
    selection::truncation::MaximizeSelector, types::fmt::Display,
};
use crate::calcs::*;

mod example;
#[cfg(test)]
mod tests;
mod calcs;

const A_STEP:usize = 100;
const NUM_CITIES: usize = 9;
const NUM_VEHICLES: usize = 4;
const HIGHEST_FITNESS: usize = 10_000;
const TOTAL_LEN: usize = 2 * NUM_VEHICLES * NUM_CITIES + 2 * NUM_VEHICLES * NUM_CITIES * NUM_CITIES;
const T:[[f64;NUM_CITIES];NUM_CITIES]=[
    [0., 2.72, 0.70, 3.78, 1.27, 3.27, 1.13, 1.3, 1.93],
    [2.72, 0., 3.27, 5.3, 3.25, 4.93, 3.08, 1.32, 3.42],
    [0.7, 3.27, 0., 3.78, 1.87, 3.5, 1.63, 1.57, 1.68],
    [3.78, 5.3, 3.78, 0., 3.38, 1.37, 3.23, 4.25, 2.32],
    [1.27, 3.25, 1.87, 3.38, 0., 3.48, 0.48, 2.03, 1.15],
    [3.27, 4.93, 3.5, 1.37, 3.48, 0., 3.12, 4.77, 3.08],
    [1.13, 3.08, 1.63, 3.23, 0.48, 3.12, 0., 1.8, 0.82],
    [1.3, 1.32, 1.57, 4.25, 2.03, 4.77, 1.8, 0., 2.17],
    [1.93, 3.42, 1.68, 2.32, 1.15, 3.08, 0.82, 2.17, 0.],
];
const DEMANDS:[usize;NUM_CITIES]=[37209, 34583, 33075, 32145, 26916, 15453, 13476, 10560, 10006];
const ALPHA:[f64;NUM_CITIES]=[1.;NUM_CITIES];
const BETA:[f64;NUM_CITIES]=[1.;NUM_CITIES];
static MAX_F1:AtomicIsize=AtomicIsize::new((f64::NEG_INFINITY)as isize);
// static MAX_F1:AtomicIsize=AtomicIsize::new(0);
static MIN_F1:AtomicIsize=AtomicIsize::new((f64::INFINITY)as isize);
// static MIN_F1:AtomicIsize=AtomicIsize::new(10000);
static MAX_F2:AtomicIsize=AtomicIsize::new((f64::NEG_INFINITY)as isize);
// static MAX_F2:AtomicIsize=AtomicIsize::new(-1000000);
static MIN_F2:AtomicIsize=AtomicIsize::new((f64::INFINITY)as isize);
// static MIN_F2:AtomicIsize=AtomicIsize::new(0);

#[derive(Debug)]
struct Parameter {
    population_size: usize,
    generation_limit: u64,
    num_individuals_per_parents: usize,
    selection_ratio: f64,
    num_crossover_points: usize,
    mutation_rate: f64,
    reinsertion_ratio: f64,
}

impl Default for Parameter {
    fn default() -> Self {
        Parameter {
            population_size: 200,
            generation_limit: 2000,
            num_individuals_per_parents: 2,
            selection_ratio: 0.7,
            num_crossover_points: 300,
            mutation_rate: 0.09,
            reinsertion_ratio: 0.7,
        }
    }
}

#[derive(Debug)]
struct Solution {
    xiko: Vec<Vec<usize>>,
    xikr: Vec<Vec<usize>>,
    yijko: Vec<Vec<Vec<bool>>>,
    yijkr: Vec<Vec<Vec<bool>>>,
}

impl Display for Solution {
    fn fmt(&self) -> String {
        let mut result = String::new();
        result.push_str("###########################################\n");
        result.push_str("##################初救阶段#################\n");
        result.push_str("###########################################\n");
        for k in 0..NUM_VEHICLES{
            result.push_str(&format!("车辆{}：\n",k+1));

            result.push_str("  配送：\n");
            let mut distribution:Vec<usize>=Vec::new();
            let mut set_off = false;
            for i in 0..NUM_CITIES{
                for j in 0..NUM_CITIES{
                    if self.yijko[k][i][j]{
                        // println!("[{}][{}][{}],{}",k,i,j,set_off);
                        if !set_off{
                            distribution.push(self.xiko[k][i]);
                            set_off=true;
                        }
                        distribution.push(self.xiko[k][j]);
                        break;
                    }
                }
            }
            result.push_str(&format!("{:?}\n",distribution));

            result.push_str("  路径：\n");
            result.push_str(&format!("{:?}\n",self.get_route_of_k_in_stage_u(k,&Stage::O)));
        }
        result.push('\n');
        result.push_str("###########################################\n");
        result.push_str("##################补救阶段#################\n");
        result.push_str("###########################################\n");
        for k in 0..NUM_VEHICLES{
            result.push_str(&format!("车辆{}：\n",k+1));

            result.push_str("  配送：\n");
            let mut distribution:Vec<usize>=Vec::new();
            let mut set_off = false;
            for i in 0..NUM_CITIES{
                for j in 0..NUM_CITIES{
                    if self.yijkr[k][i][j]{
                        // println!("[{}][{}][{}],{}",k,i,j,set_off);
                        if !set_off{
                            distribution.push(self.xikr[k][i]);
                            set_off=true;
                        }
                        distribution.push(self.xikr[k][j]);
                        break;
                    }
                }
            }
            result.push_str(&format!("{:?}\n",distribution));

            result.push_str("  路径：\n");
            result.push_str(&format!("{:?}\n",self.get_route_of_k_in_stage_u(k,&Stage::R)));
        }


        result
    }
}

type Genome = Vec<u8>;

fn decode_x(u: &u8) -> usize {
    *u as usize * A_STEP
}
fn decode_y(u: &u8) -> bool {
    *u > 0b1111_1111 / 2
}

trait AsPhenotype {
    fn as_solution(&self) -> Solution;
}

impl AsPhenotype for Genome {
    fn as_solution(&self) -> Solution {
        let mut xiko: Vec<Vec<usize>> = Vec::new();
        let mut xikr: Vec<Vec<usize>> = Vec::new();
        let mut yijko: Vec<Vec<Vec<bool>>> = Vec::new();
        let mut yijkr: Vec<Vec<Vec<bool>>> = Vec::new();
        for i in 0..NUM_VEHICLES {
            xiko.push(Vec::new());
            for j in 0..NUM_CITIES {
                xiko[i].push(decode_x(&self[i * NUM_CITIES + j]));
            }
        }
        for i in 0..NUM_VEHICLES {
            xikr.push(Vec::new());
            for j in 0..NUM_CITIES {
                xikr[i].push(decode_x(
                    &self[NUM_VEHICLES * NUM_CITIES + i * NUM_CITIES + j],
                ));
            }
        }

        for k in 0..NUM_VEHICLES {
            yijko.push(Vec::new());
            for i in 0..NUM_CITIES {
                yijko[k].push(Vec::new());
                for j in 0..NUM_CITIES {
                    yijko[k][i].push(decode_y(
                        &self[2 * NUM_VEHICLES * NUM_CITIES
                            + k * NUM_CITIES * NUM_CITIES
                            + i * NUM_CITIES
                            + j],
                    ));
                }
            }
        }
        for k in 0..NUM_VEHICLES {
            yijkr.push(Vec::new());
            for i in 0..NUM_CITIES {
                yijkr[k].push(Vec::new());
                for j in 0..NUM_CITIES {
                    yijkr[k][i].push(decode_y(
                        &self[2 * NUM_VEHICLES * NUM_CITIES
                            + NUM_VEHICLES * NUM_CITIES * NUM_CITIES
                            + k * NUM_CITIES * NUM_CITIES
                            + i * NUM_CITIES
                            + j],
                    ));
                }
            }
        }

        Solution {
            xiko,
            xikr,
            yijko,
            yijkr,
        }
    }
}

#[derive(Clone, Debug)]
struct FitnessCalc;

impl FitnessFunction<Genome, usize> for FitnessCalc {
    // TODO: 适应度函数
    fn fitness_of(&self, genome: &Genome) -> usize {
        let mut fitness: usize = HIGHEST_FITNESS;
        let solution = genome.as_solution();
        let uniformalized_f=solution.uniformalized_f();
        // dbg!(&uniformalized_f);
        let subtraction = uniformalized_f*(HIGHEST_FITNESS as f64);
        // dbg!(&subtraction);
        fitness-=subtraction as usize;
        fitness
    }

    fn average(&self, fitness_values: &[usize]) -> usize {
        fitness_values.iter().sum::<usize>() / fitness_values.len()
    }

    fn highest_possible_fitness(&self) -> usize {
        HIGHEST_FITNESS
    }

    fn lowest_possible_fitness(&self) -> usize {
        0
    }
}


fn main() {


    let params = Parameter::default();

    let initial_population: Population<Genome> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(TOTAL_LEN, 0, 0b1111_1111))
        .of_size(params.population_size)
        .uniform_at_random();

    for individual in initial_population.individuals(){
        individual.as_solution().f1();
        individual.as_solution().f2();
    }

    let mut sim = simulate(
        genetic_algorithm()
            .with_evaluation(FitnessCalc)
            .with_selection(MaximizeSelector::new(
                params.selection_ratio,
                params.num_individuals_per_parents,
            ))
            .with_crossover(MultiPointCrossBreeder::new(params.num_crossover_points))
            .with_mutation(RandomValueMutator::new(
                params.mutation_rate,
                0,
                0b1111_1111,
            ))
            .with_reinsertion(ElitistReinserter::new(
                FitnessCalc,
                true,
                params.reinsertion_ratio,
            ))
            .with_initial_population(initial_population)
            .build(),
    )
    .until(or(
        FitnessLimit::new(FitnessCalc.highest_possible_fitness()),
        GenerationLimit::new(params.generation_limit),
    ))
    .build();

    println!("开始进化...");

    loop {
        match sim.step() {
            Ok(SimResult::Intermediate(step)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                println!(
                    "generation: {}, average_fitness: {}, \
                    best_fitness: {}, duration: {}, processing_time: {}",
                    step.iteration,
                    evaluated_population.average_fitness(),
                    best_solution.solution.fitness,
                    step.duration.fmt(),
                    step.processing_time.fmt(),
                    );
                // println!("uniformalized_f: {}",best_solution.solution.genome.as_solution().uniformalized_f());
                // dbg!(&best_solution.solution.genome.as_solution().xiko[0]);
            },
            Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                let best_solution=step.result.best_solution;
                println!("{}",stop_reason);
                println!(
                    "Final result after {}: generation: {}, \
                    best solution with fitness {} found in generation {}, processing_time: {}",
                    duration.fmt(),
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time.fmt(),
                    );
                println!("best solution: \n{}",best_solution.solution.genome.as_solution().fmt());
                // println!("{}",best_solution.solution.genome.as_solution().uniformalized_f());
                // println!("{}, {}", best_solution.solution.genome.as_solution().f1(),best_solution.solution.genome.as_solution().f2());

                break;
            },
            Err(e) => {
                println!("{e}");
                break;
            },
        }
    }
}
