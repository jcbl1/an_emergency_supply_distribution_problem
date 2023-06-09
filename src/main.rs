use std::{
    eprintln, fs, println, process,
    sync::atomic::{AtomicIsize, AtomicUsize, Ordering},
};

use crate::calcs::*;
use genevo::{
    operator::prelude::RandomValueMutator, population::ValueEncodedGenomeBuilder, prelude::*,
    recombination::discrete::MultiPointCrossBreeder, reinsertion::elitist::ElitistReinserter,
    selection::truncation::MaximizeSelector, types::fmt::Display,
};

use xlsxwriter::prelude::*;

use clap::{Arg, ArgAction, ArgMatches, Command};

mod calcs;
mod example;
#[cfg(test)]
mod tests;

const A_STEP: usize = 1000;
const NUM_CITIES: usize = 9;
const NUM_VEHICLES: usize = 4;
const HIGHEST_FITNESS: usize = 1_000_000_000;
const TOTAL_LEN: usize = 2 * NUM_VEHICLES * NUM_CITIES + 24;
const T: [[f64; NUM_CITIES]; NUM_CITIES] = [
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
// 最长路线：3->1->5->7->8->0->4->2->6->3；总时长27.1
const DEMANDS: [usize; NUM_CITIES] = [
    37209, 34583, 33075, 32145, 26916, 15453, 13476, 10560, 10006,
];
const ALPHA: [f64; NUM_CITIES] = [1.; NUM_CITIES];
const BETA: [f64; NUM_CITIES] = [1.; NUM_CITIES];
// static MAX_F1: AtomicIsize = AtomicIsize::new((f64::NEG_INFINITY) as isize);
// static MAX_F1: AtomicIsize = AtomicIsize::new(800isize);
// const MAX_F1: f64 = 54.2f64;
const MAX_F1: f64 = 65f64;
// static MIN_F1: AtomicIsize = AtomicIsize::new((f64::INFINITY) as isize);
// static MIN_F1: AtomicIsize = AtomicIsize::new(400isize);
const MIN_F1: f64 = 0f64;
// static MAX_F2: AtomicIsize = AtomicIsize::new((f64::NEG_INFINITY) as isize);
static MAX_F2: AtomicIsize = AtomicIsize::new(0isize);
// static MIN_F2: AtomicIsize = AtomicIsize::new((f64::INFINITY) as isize);
static MIN_F2: AtomicIsize = AtomicIsize::new(-25239500isize);
static MAX_R8: AtomicUsize = AtomicUsize::new(650703007);

static UNTIL_NEXT_STAGE: AtomicUsize = AtomicUsize::new(0);

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
        // let len = 2 * NUM_VEHICLES * NUM_CITIES + 2 * NUM_VEHICLES * NUM_CITIES * NUM_CITIES;
        let len = 2 * NUM_VEHICLES * NUM_CITIES + 24;
        Parameter {
            population_size: (100. * (len as f64).ln()) as usize,
            generation_limit: 400,
            num_individuals_per_parents: 2,
            selection_ratio: 0.7,
            // selection_ratio: 0.8,
            num_crossover_points: len / 6,
            mutation_rate: 0.5 / (len as f64).ln(),
            // mutation_rate: 0.5,
            reinsertion_ratio: 0.7,
            // reinsertion_ratio: 0.8,
        }
    }
}

#[derive(Debug)]
struct Solution {
    xiko: Vec<Vec<usize>>,
    xikr: Vec<Vec<usize>>,
    u8s: Vec<u8>,
    // yijko: Vec<Vec<Vec<bool>>>,
    // yijkr: Vec<Vec<Vec<bool>>>,
    parts: Vec<f64>,
    totr: (f64, f64),
    routes_o: Vec<Vec<usize>>,
    routes_r: Vec<Vec<usize>>,
}

impl Display for Solution {
    fn fmt(&self) -> String {
        let mut result = String::new();
        result.push_str("###########################################\n");
        result.push_str("##################初救阶段#################\n");
        result.push_str("###########################################\n");
        for k in 0..NUM_VEHICLES {
            result.push_str(&format!("车辆{}：\n", k + 1));

            result.push_str("  配送：\n");
            let distribution = &self.xiko[k];
            // let mut distribution: Vec<usize> = Vec::new();
            // let mut set_off = false;
            // for i in 0..NUM_CITIES {
            //     for j in 0..NUM_CITIES {
            //         if self.yijko[k][i][j] {
            //             // println!("[{}][{}][{}],{}",k,i,j,set_off);
            //             if !set_off {
            //                 distribution.push(self.xiko[k][i]);
            //                 set_off = true;
            //             }
            //             distribution.push(self.xiko[k][j]);
            //             break;
            //         }
            //     }
            // }
            result.push_str(&format!("{:?}\n", distribution));

            result.push_str("  路径：\n");
            result.push_str(&format!("{:?}\n", self.routes_o[k]));
        }
        result.push('\n');
        result.push_str("###########################################\n");
        result.push_str("##################补救阶段#################\n");
        result.push_str("###########################################\n");
        for k in 0..NUM_VEHICLES {
            result.push_str(&format!("车辆{}：\n", k + 1));

            result.push_str("  配送：\n");
            let distribution = &self.xikr[k];
            // let mut distribution: Vec<usize> = Vec::new();
            // let mut set_off = false;
            // for i in 0..NUM_CITIES {
            //     for j in 0..NUM_CITIES {
            //         if self.yijkr[k][i][j] {
            //             // println!("[{}][{}][{}],{}",k,i,j,set_off);
            //             if !set_off {
            //                 distribution.push(self.xikr[k][i]);
            //                 set_off = true;
            //             }
            //             distribution.push(self.xikr[k][j]);
            //             break;
            //         }
            //     }
            // }
            result.push_str(&format!("{:?}\n", distribution));

            result.push_str("  路径：\n");
            result.push_str(&format!("{:?}\n", self.routes_r[k]));
        }

        result
    }
}

type Genome = Vec<u8>;

fn decode_x(u: &u8) -> usize {
    *u as usize * A_STEP
}
// fn decode_y(u: &u8) -> bool {
//     *u > 0b1111_1111 / 2
// }

trait AsPhenotype {
    fn as_solution(&self) -> Solution;
}

impl AsPhenotype for Genome {
    fn as_solution(&self) -> Solution {
        let mut xiko: Vec<Vec<usize>> = Vec::new();
        let mut xikr: Vec<Vec<usize>> = Vec::new();
        // let mut yijko: Vec<Vec<Vec<bool>>> = Vec::new();
        // let mut yijkr: Vec<Vec<Vec<bool>>> = Vec::new();
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

        let start = 2 * NUM_VEHICLES * NUM_CITIES;
        let u8s = Vec::from(&self[start..(start + 24)]);
        // let routes=takes_24_u8s_and_returns_8_routes(u8s.clone()).unwrap();
        // let routes_o=Vec::from(&routes[0..4]);
        // let routes_r=Vec::from(&routes[4..]);
        // assert_eq!(routes_o.len(),4);
        // assert_eq!(routes_r.len(),4);
        // drop(routes);

        // for k in 0..NUM_VEHICLES {
        //     yijko.push(Vec::new());
        //     for i in 0..NUM_CITIES {
        //         yijko[k].push(Vec::new());
        //         for j in 0..NUM_CITIES {
        //             yijko[k][i].push(decode_y(
        //                 &self[2 * NUM_VEHICLES * NUM_CITIES
        //                     + k * NUM_CITIES * NUM_CITIES
        //                     + i * NUM_CITIES
        //                     + j],
        //             ));
        //         }
        //     }
        // }
        // for k in 0..NUM_VEHICLES {
        //     yijkr.push(Vec::new());
        //     for i in 0..NUM_CITIES {
        //         yijkr[k].push(Vec::new());
        //         for j in 0..NUM_CITIES {
        //             yijkr[k][i].push(decode_y(
        //                 &self[2 * NUM_VEHICLES * NUM_CITIES
        //                     + NUM_VEHICLES * NUM_CITIES * NUM_CITIES
        //                     + k * NUM_CITIES * NUM_CITIES
        //                     + i * NUM_CITIES
        //                     + j],
        //             ));
        //         }
        //     }
        // }

        let mut solution = Solution {
            xiko,
            xikr,
            u8s,
            // yijko,
            // yijkr,
            parts: vec![0., 0., 0., 0.],
            totr: (0., 0.),
            // routes_o: vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            // routes_r: vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            routes_o: Vec::new(),
            routes_r: Vec::new(),
        };
        solution.uniformalized_f();
        // solution.update_routes();
        // solution.update_totr();
        solution
    }
}

#[derive(Clone, Debug)]
struct FitnessCalc;

impl FitnessFunction<Genome, usize> for FitnessCalc {
    // TODO: 适应度函数
    fn fitness_of(&self, genome: &Genome) -> usize {
        let mut fitness: usize = HIGHEST_FITNESS;
        let mut solution = genome.as_solution();
        let uniformalized_f = solution.uniformalized_f();
        // dbg!(&uniformalized_f);
        let subtraction = uniformalized_f * (HIGHEST_FITNESS as f64);
        // dbg!(&subtraction);
        fitness -= subtraction as usize;
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

fn parse_matches() -> ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("毕业论文模型遗传算法实现.")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("设置存放xlsx文件的文件夹")
                .default_value("./output")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("generation_limit")
                .short('g')
                .long("generation-limit")
                .help("最大代数")
                .default_value("400")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("population_size")
                .short('p')
                .long("population-size")
                .help("种群规模，默认值为100 * ln(NUM_OF_BYTES)")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Debug mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stage_save")
                .short('s')
                .long("stage-save")
                .help("是否需要记录阶段性结果")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("weights")
                .short('w')
                .long("weights")
                .help("设置各部分权重")
                .default_value("1,1,1,1")
                .action(ArgAction::Set),
        )
        .get_matches()
}

fn main() {
    let matches = parse_matches();
    let debug_mode = matches.get_flag("debug");
    if debug_mode {
        dbg!("Processing in debug mode.");
    }
    //权值/分目标的个数
    let mut len = 0;
    if let Some(weights) = matches.get_one::<String>("weights") {
        //权值个数由内部WEIGHTS的长度决定
        len = WEIGHTS.len();
        let mut weights: Vec<_> = weights
            .split(',')
            .map(|w| w.parse::<f64>().expect("Error parsing weights"))
            .collect();
        if weights.len() != len {
            eprintln!("Error: 权值个数不为{}", len);
            process::exit(1);
        }
        //计算真正使用到的权值，即和为1的权值并存储在WEIGHTS中
        let sum = weights.iter().sum::<f64>();
        for i in 0..len {
            weights[i] = weights[i] / sum;
        }
        for i in 0..len {
            WEIGHTS[i].store((weights[i] * 10000f64) as usize, Ordering::Relaxed);
        }
        if debug_mode {
            dbg!(&WEIGHTS);
        }
    }
    // 设置存储路径
    let output = match matches.get_one::<String>("output") {
        Some(output) => output,
        None => "./output",
    };
    //设置文件名
    let uuid_ = uuid::Uuid::new_v4();
    //创建文件夹
    fs::create_dir(output).unwrap_or_else(|e| match e.kind() {
        std::io::ErrorKind::AlreadyExists => (),
        other => panic!("{other}"),
    });
    //
    let workbook =
        Workbook::new(&format!("{}/{}.xlsx", output, uuid_)).expect("Error creating file");
    println!("结果将会存放在{}/{}.xlsx.", output, uuid_);
    println!("正在初始化xlsx文件...");
    let mut format_label = Format::new();
    format_label
        .set_bold()
        .set_align(FormatAlignment::Center)
        .set_vertical_align(FormatVerticalAlignment::VerticalCenter);
    //存放各代适应度信息的表
    let mut fitness_sheet = workbook
        .add_worksheet(Some("Fitness"))
        .expect("Error creating sheet");
    fitness_sheet
        .write_string(0, 0, "gen", Some(&format_label))
        .expect("Error write_string");
    fitness_sheet
        .write_string(0, 1, "highest_fitness", Some(&format_label))
        .expect("Error write_string");
    fitness_sheet
        .write_string(0, 2, "average_fitness", Some(&format_label))
        .expect("Error write_string");
    fitness_sheet
        .write_string(0, 3, "lowest_fitness", Some(&format_label))
        .expect("Error write_string");
    for i in 0..len {
        fitness_sheet
            .write_string(
                0,
                (4 + i) as u16,
                &format!("parts[{}]", i),
                Some(&format_label),
            )
            .expect("Error write_string");
    }
    fitness_sheet
        .write_string(0, (4 + len) as u16, "fitnesses", Some(&format_label))
        .expect("Error write_string");
    let write_gen = |fitness_sheet: &mut Worksheet,
                     gen: u64,
                     highest_fitness: usize,
                     average_fitness: usize,
                     lowest_fitness: usize,
                     parts: &Vec<f64>,
                     fitnesses: &[usize]| {
        let (gen, gen_value, highest_fitness, average_fitness, lowest_fitness) = (
            gen as u32,
            gen as f64,
            highest_fitness as f64,
            average_fitness as f64,
            lowest_fitness as f64,
        );
        fitness_sheet
            .write_number(gen, 0, gen_value, None)
            .expect("Error write_number");
        fitness_sheet
            .write_number(gen, 1, highest_fitness, None)
            .expect("Error write_number");
        fitness_sheet
            .write_number(gen, 2, average_fitness, None)
            .expect("Error write_number");
        fitness_sheet
            .write_number(gen, 3, lowest_fitness, None)
            .expect("Error write_number");
        for i in 0..len {
            fitness_sheet
                .write_number(gen, (4 + i) as u16, parts[i], None)
                .expect("Error write_number");
        }
        for col in (4 + len)..(fitnesses.len() + 4 + len) {
            fitness_sheet
                .write_number(gen, col as u16, fitnesses[col - 4 - len] as f64, None)
                .expect("Error write_number");
        }
    };
    // 存储最终结果的表
    let mut final_result_sheet = workbook
        .add_worksheet(Some("Final Results"))
        .expect("Error creating sheet");
    final_result_sheet
        .write_string(0, 0, "stop_reason", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_string(1, 0, "best_solution", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_string(1, 1, "初救阶段", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_string(10, 1, "补救阶段", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_string(1, 2, "受灾点", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_string(10, 2, "受灾点", Some(&format_label))
        .expect("Error write_string");
    for u in 0..2 {
        for col in 3..12 {
            final_result_sheet
                .write_number((1 + u * 9) as u32, col, (col - 2) as f64, None)
                .expect("Error write_number");
        }
        for k in 0..NUM_VEHICLES {
            final_result_sheet
                .merge_range(
                    (2 + k * 2 + u * 9) as u32,
                    1,
                    (3 + k * 2 + u * 9) as u32,
                    1,
                    &format!("车辆{}", k + 1),
                    Some(&format_label),
                )
                .expect("Error merge_range");
            final_result_sheet
                .write_string((2 + k * 2 + u * 9) as u32, 2, "分配", Some(&format_label))
                .expect("Error write_string");
            final_result_sheet
                .write_string((3 + k * 2 + u * 9) as u32, 2, "路径", Some(&format_label))
                .expect("Error write_string");
        }
    }

    final_result_sheet
        .write_string(21, 3, "t_o", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_string(21, 5, "t_o", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_string(22, 2, "受灾点", Some(&format_label))
        .expect("Error write_string");
    for i in 1..=NUM_CITIES {
        final_result_sheet
            .write_number(22, (i + 2) as u16, i as f64, None)
            .expect("Error write_number");
    }
    final_result_sheet
        .write_string(23, 2, "受灾人数", Some(&format_label))
        .expect("Error write_string");
    for i in 0..NUM_CITIES {
        final_result_sheet
            .write_number(23, (i + 3) as u16, DEMANDS[i] as f64, None)
            .expect("Error write_number");
    }
    final_result_sheet
        .write_string(25, 2, "需求o", Some(&format_label))
        .expect("Error write_string");
    for i in 0..NUM_CITIES {
        final_result_sheet
            .write_formula(25, (i + 3) as u16, "=D24+D24*POWER($E$22,1.08)/1.66", None)
            .expect("Error write_formula");
    }
    final_result_sheet
        .write_string(26, 2, "需求r", Some(&format_label))
        .expect("Error write_string");
    for i in 0..NUM_CITIES {
        final_result_sheet
            .write_formula(26, (i + 3) as u16, "=D24*POWER($G$22,1.08)/1.66", None)
            .expect("Error write_formula");
    }
    final_result_sheet
        .write_string(27, 2, "总需求", Some(&format_label))
        .expect("Error write_string");
    for i in 0..NUM_CITIES {
        final_result_sheet
            .write_formula(27, (i + 3) as u16, "=D26+D27", None)
            .expect("Error write_formula");
    }
    final_result_sheet
        .write_string(28, 2, "供给o", Some(&format_label))
        .unwrap();
    final_result_sheet
        .write_string(29, 2, "供给r", Some(&format_label))
        .unwrap();
    final_result_sheet
        .write_string(30, 2, "总供给", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_formula(30, 3, "=SUM(D29:D30)", None)
        .unwrap();
    final_result_sheet
        .write_string(31, 2, "差", Some(&format_label))
        .expect("Error write_string");
    for i in 0..NUM_CITIES {
        final_result_sheet
            .write_formula(31, (i + 3) as u16, "=ABS(D28-D31)", None)
            .expect("Error write_formula");
    }
    final_result_sheet
        .write_string(30, 12, "和", Some(&format_label))
        .expect("Error write_string");
    final_result_sheet
        .write_formula(31, 12, "=SUM(D32:L32)", None)
        .expect("Error write_formula");

    final_result_sheet
        .write_string(32, 2, "U_i_o", Some(&format_label))
        .unwrap();
    for i in 0..NUM_CITIES {
        final_result_sheet.write_formula(32, (i+3)as u16, "=D29-SUM(MAX($D$29-D29,0),MAX($E$29-D29,0),MAX($F$29-D29,0),MAX($G$29-D29,0),MAX($H$29-D29,0),MAX($I$29-D29,0),MAX($J$29-D29,0),MAX($K$29-D29,0),MAX($L$29-D29,0))/8-SUM(MAX(D29-$D$29,0),MAX(D29-$E$29,0),MAX(D29-$F$29,0),MAX(D29-$G$29,0),MAX(D29-$H$29,0),MAX(D29-$I$29,0),MAX(D29-$J$29,0),MAX(D29-$K$29,0),MAX(D29-$L$29,0))/8", None).unwrap();
    }
    final_result_sheet
        .write_formula(32, 12, "=SUM(D33:L33)", None)
        .unwrap();
    final_result_sheet
        .write_string(33, 2, "U_i_r", Some(&format_label))
        .unwrap();
    for i in 0..NUM_CITIES {
        final_result_sheet.write_formula(33, (i+3)as u16, "=D30-SUM(MAX($D$30-D30,0),MAX($E$30-D30,0),MAX($F$30-D30,0),MAX($G$30-D30,0),MAX($H$30-D30,0),MAX($I$30-D30,0),MAX($J$30-D30,0),MAX($K$30-D30,0),MAX($L$30-D30,0))/8-SUM(MAX(D30-$D$30,0),MAX(D30-$E$30,0),MAX(D30-$F$30,0),MAX(D30-$G$30,0),MAX(D30-$H$30,0),MAX(D30-$I$30,0),MAX(D30-$J$30,0),MAX(D30-$K$30,0),MAX(D30-$L$30,0))/8", None).unwrap();
    }
    final_result_sheet
        .write_formula(33, 12, "=SUM(D34:L34)", None)
        .unwrap();

    final_result_sheet
        .write_string(35, 11, "U和之相反数", Some(&format_label))
        .unwrap();
    final_result_sheet
        .write_formula(35, 12, "=-SUM(M33:M34)", None)
        .unwrap();

    final_result_sheet
        .write_string(36, 11, "max_f2", Some(&format_label))
        .unwrap();
    final_result_sheet
        .write_string(37, 11, "min_f2", Some(&format_label))
        .unwrap();
    final_result_sheet
        .write_string(38, 11, "f2", Some(&format_label))
        .unwrap();
    final_result_sheet
        .write_formula(38, 12, "=(M36-$M$38)/($M$37-$M$38)", None)
        .unwrap();

    let write_final =
        |final_result_sheet: &mut Worksheet, stop_reason: &str, solution: &Solution| {
            final_result_sheet
                .merge_range(0, 1, 0, 11, stop_reason, None)
                .expect("Error merge_range");
            for k in 0..NUM_VEHICLES {
                for i in 0..NUM_CITIES {
                    final_result_sheet
                        .write_number(
                            (2 + 2 * k) as u32,
                            (i + 3) as u16,
                            (solution.xiko[k][i]) as f64,
                            None,
                        )
                        .expect("Error write_number");
                    final_result_sheet
                        .write_number(
                            (9 + 2 + 2 * k) as u32,
                            (i + 3) as u16,
                            (solution.xikr[k][i]) as f64,
                            None,
                        )
                        .expect("Error write_number");
                }
                let route_o = solution.routes_o[k].clone();
                let route_r = solution.routes_r[k].clone();
                final_result_sheet
                    .write_string((3 + 2 * k) as u32, 3, &format!("{:?}", route_o), None)
                    .expect("Error write_string");
                final_result_sheet
                    .write_string((9 + 3 + 2 * k) as u32, 3, &format!("{:?}", route_r), None)
                    .expect("Error write_string");
            }

            final_result_sheet
                .write_number(21, 4, solution.totr.0, None)
                .expect("Error write_number");
            final_result_sheet
                .write_number(21, 6, solution.totr.1, None)
                .unwrap();
            for i in 0..NUM_CITIES {
                let xio = solution.delivered_to_i_in_stage_u(i, &Stage::O);
                let xir = solution.delivered_to_i_in_stage_u(i, &Stage::R);
                final_result_sheet
                    .write_number(28, (i + 3) as u16, xio, None)
                    .unwrap();
                final_result_sheet
                    .write_number(29, (i + 3) as u16, xir, None)
                    .unwrap();
            }
            let max_f2 = MAX_F2.load(Ordering::Relaxed) as f64;
            let min_f2 = MIN_F2.load(Ordering::Relaxed) as f64;
            final_result_sheet
                .write_number(36, 12, max_f2, None)
                .unwrap();
            final_result_sheet
                .write_number(37, 12, min_f2, None)
                .unwrap();
        };

    //存储朴素结果以供调试
    let mut plain_result_sheet = workbook
        .add_worksheet(Some("Plain Result"))
        .expect("Error creating sheet");
    plain_result_sheet
        .write_string(0, 0, "xiko", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(5, 0, "xikr", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(10, 0, "yijko,k=0", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(20, 0, "yijko,k=1", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(30, 0, "yijko,k=2", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(40, 0, "yijko,k=3", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(50, 0, "yijkr,k=0", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(60, 0, "yijkr,k=1", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(70, 0, "yijkr,k=2", Some(&format_label))
        .expect("Error write_string");
    plain_result_sheet
        .write_string(80, 0, "yijkr,k=3", Some(&format_label))
        .expect("Error write_string");
    let write_plain_results = |plain_result_sheet: &mut Worksheet, solution: &Solution| {
        for k in 0..NUM_VEHICLES {
            for i in 0..NUM_CITIES {
                plain_result_sheet
                    .write_number((k + 1) as u32, i as u16, solution.xiko[k][i] as f64, None)
                    .expect("Error write_number");
                plain_result_sheet
                    .write_number((k + 6) as u32, i as u16, solution.xikr[k][i] as f64, None)
                    .expect("Error write_number");
            }
        }
        //TODO:存储u8s
        // for k in 0..NUM_VEHICLES {
        //     for i in 0..NUM_CITIES {
        //         for j in 0..NUM_CITIES {
        //             plain_result_sheet
        //                 .write_boolean(
        //                     (11 + k * 10 + i) as u32,
        //                     j as u16,
        //                     solution.yijko[k][i][j],
        //                     None,
        //                 )
        //                 .expect("Error write_boolean");
        //             plain_result_sheet
        //                 .write_boolean(
        //                     (51 + k * 10 + i) as u32,
        //                     j as u16,
        //                     solution.yijkr[k][i][j],
        //                     None,
        //                 )
        //                 .expect("Error write_boolean");
        //         }
        //     }
        // }
    };

    let write_stage = |workbook: &Workbook, gen: u64, solution: &Solution| {
        let mut sheet = workbook
            .add_worksheet(Some(&format!("gen {gen}")))
            .expect("Error creating sheet");
        sheet
            .write_string(0, 0, "xiko", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(5, 0, "xikr", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(10, 0, "yijko,k=0", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(20, 0, "yijko,k=1", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(30, 0, "yijko,k=2", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(40, 0, "yijko,k=3", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(50, 0, "yijkr,k=0", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(60, 0, "yijkr,k=1", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(70, 0, "yijkr,k=2", Some(&format_label))
            .expect("Error write_string");
        sheet
            .write_string(80, 0, "yijkr,k=3", Some(&format_label))
            .expect("Error write_string");

        write_plain_results(&mut sheet, &solution);
    };

    //默认参数
    let mut params = Parameter::default();
    //更新代数
    if let Some(generation_limit) = matches.get_one::<String>("generation_limit") {
        let generation_limit = generation_limit
            .parse::<u64>()
            .expect("Error parsing generation_limit");
        params = Parameter {
            generation_limit,
            ..params
        };
    }
    //更新种群规模
    if let Some(population_size) = matches.get_one::<String>("population_size") {
        let population_size = population_size
            .parse::<usize>()
            .expect("Error parsing population_size");
        params = Parameter {
            population_size,
            ..params
        };
    }
    println!("参数：{:#?}", params);

    let stage_save = matches.get_flag("stage_save");

    let initial_population: Population<Genome> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(TOTAL_LEN, 0, 0b1111_1111))
        .of_size(params.population_size)
        .uniform_at_random();

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

    println!("开始进化.");

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
                write_gen(
                    &mut fitness_sheet,
                    step.iteration,
                    best_solution.solution.fitness,
                    *evaluated_population.average_fitness(),
                    *evaluated_population.lowest_fitness(),
                    &best_solution.solution.genome.as_solution().parts,
                    evaluated_population.fitness_values(),
                );
                if stage_save {
                    let count = UNTIL_NEXT_STAGE.load(Ordering::Relaxed);
                    if count > 1000 {
                        write_stage(
                            &workbook,
                            step.iteration,
                            &best_solution.solution.genome.as_solution(),
                        );
                        UNTIL_NEXT_STAGE.store(0, Ordering::Relaxed);
                    } else {
                        UNTIL_NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
                    }
                }
                // println!("uniformalized_f: {}",best_solution.solution.genome.as_solution().uniformalized_f());
                // dbg!(&best_solution.solution.genome.as_solution().xiko[0]);
            }
            Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                let best_solution = step.result.best_solution;
                println!("{}", stop_reason);
                println!(
                    "Final result after {}: generation: {}, \
                    best solution with fitness {} found in generation {}, processing_time: {}",
                    duration.fmt(),
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time.fmt(),
                );
                // println!(
                //     "best solution: \n{}",
                //     best_solution.solution.genome.as_solution().fmt()
                // );

                write_gen(
                    &mut fitness_sheet,
                    step.iteration,
                    best_solution.solution.fitness,
                    *step.result.evaluated_population.average_fitness(),
                    *step.result.evaluated_population.lowest_fitness(),
                    &best_solution.solution.genome.as_solution().parts,
                    &[],
                );
                write_final(
                    &mut final_result_sheet,
                    &stop_reason,
                    &best_solution.solution.genome.as_solution(),
                );
                write_plain_results(
                    &mut plain_result_sheet,
                    &best_solution.solution.genome.as_solution(),
                );
                // println!("uniformalized_f: {}, fitness: {}",best_solution.solution.genome.as_solution().uniformalized_f(), best_solution.solution.fitness);
                // let calc = FitnessCalc;
                // println!("再算一遍fitness：{}",calc.fitness_of(&best_solution.solution.genome));
                // println!("{}, {}", best_solution.solution.genome.as_solution().f1(),best_solution.solution.genome.as_solution().f2());

                break;
            }
            Err(e) => {
                println!("{e}");
                break;
            }
        }
    }

    // Post work
    // 写入文件
    if let Err(e) = workbook.close() {
        eprintln!("{e}");
        process::exit(1);
    } else {
        println!("结果存放在{}/{}.xlsx. 别忘了查看啊", output, uuid_);
    }
}
