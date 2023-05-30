use std::error::Error;
use std::{fmt::Display, ops::Deref};

use clap::{Arg, Command};

#[derive(Debug)]
struct Next(usize);
impl Deref for Next {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Next {
    fn check(&mut self, result: &Vec<usize>) {
        let mut sorted = result.clone();
        sorted.sort();
        for elem in sorted {
            if self.0 >= elem {
                self.0 += 1;
            }
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    LengthError,
    IndexOutOfMapError,
}

#[derive(Debug)]
struct MyError {
    message: String,
    kind: ErrorKind,
}

impl MyError {
    fn new(kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::LengthError => Self {
                message: String::from("Length not correct"),
                kind,
            },
            ErrorKind::IndexOutOfMapError => Self {
                message: String::from("index out of map"),
                kind,
            },
        }
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        Ok(())
    }
}

fn fact(n: usize) -> usize {
    if n < 2 {
        return 1;
    }
    return n * fact(n - 1);
}

fn arrangement(m: usize, n: usize) -> usize {
    let upper = fact(n);
    let lower = fact(n - m);
    upper / lower
}
#[test]
fn test_arrangement() {
    for i in 0..=9 {
        print!("{}", arrangement(i, 9));
        if i != 9 {
            print!(", ");
        }
    }
    println!();
}

impl Error for MyError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.message
    }
}

fn convert(index: usize) -> Option<Vec<usize>> {
    match index {
        0 => Some(vec![]),
        1..=9 => Some(vec![index as usize - 1]),
        10..=81 => {
            let mut result = Vec::new();
            result.push((index - 10) / 8);
            let mut next = (index - 10) % 8;
            if next >= result[0] {
                next += 1;
            }
            result.push(next);
            Some(result)
        }
        82..=585 => {
            let mut result = Vec::new();
            result.push((index - 82) / 56);
            let mut next = (index - 82) % 56 / 7;
            if next >= result[0] {
                next += 1;
            }
            result.push(next);
            let mut next = (index - 82) % 56 % 7;
            let mut sorted = result.clone();
            sorted.sort();
            for elem in sorted {
                if next >= elem {
                    next += 1;
                }
            }
            result.push(next);
            Some(result)
        }
        586..=3609 => {
            let mut result = Vec::new();
            let next = Next((index - 586) / (8 * 7 * 6));
            result.push(*next);
            let mut next = Next((index - 586) % (8 * 7 * 6) / (7 * 6));
            next.check(&result);
            result.push(*next);
            let mut next = Next((index - 586) % (8 * 7 * 6) % (7 * 6) / 6);
            next.check(&result);
            result.push(*next);
            let mut next = Next((index - 586) % (8 * 7 * 6) % (7 * 6) % 6);
            next.check(&result);
            result.push(*next);

            Some(result)
        }
        3610..=18729 => {
            let index = index - 3610;
            let mut result: Vec<usize> = Vec::new();
            let next = Next(index / (8 * 7 * 6 * 5));
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5) / (7 * 6 * 5));
            next.check(&result);
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5) % (7 * 6 * 5) / (6 * 5));
            next.check(&result);
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5) % (7 * 6 * 5) % (6 * 5) / 5);
            next.check(&result);
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5) % (7 * 6 * 5) % (6 * 5) % 5);
            next.check(&result);
            result.push(*next);

            Some(result)
        }
        18730..=79209 => {
            let index = index - 18730;
            let mut result: Vec<usize> = Vec::new();
            let next = Next(index / (8 * 7 * 6 * 5 * 4));
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5 * 4) / (7 * 6 * 5 * 4));
            next.check(&result);
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5 * 4) % (7 * 6 * 5 * 4) / (6 * 5 * 4));
            next.check(&result);
            result.push(*next);
            let mut next =
                Next(index % (8 * 7 * 6 * 5 * 4) % (7 * 6 * 5 * 4) % (6 * 5 * 4) / (5 * 4));
            next.check(&result);
            result.push(*next);
            let mut next =
                Next(index % (8 * 7 * 6 * 5 * 4) % (7 * 6 * 5 * 4) % (6 * 5 * 4) % (5 * 4) / 4);
            next.check(&result);
            result.push(*next);
            let mut next =
                Next(index % (8 * 7 * 6 * 5 * 4) % (7 * 6 * 5 * 4) % (6 * 5 * 4) % (5 * 4) % 4);
            next.check(&result);
            result.push(*next);

            Some(result)
        }
        79210..=260649 => {
            let index = index - 79210;
            let mut result: Vec<usize> = Vec::new();
            let next = Next(index / (8 * 7 * 6 * 5 * 4 * 3));
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5 * 4 * 3) / (7 * 6 * 5 * 4 * 3));
            next.check(&result);
            result.push(*next);
            let mut next =
                Next(index % (8 * 7 * 6 * 5 * 4 * 3) % (7 * 6 * 5 * 4 * 3) / (6 * 5 * 4 * 3));
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index % (8 * 7 * 6 * 5 * 4 * 3) % (7 * 6 * 5 * 4 * 3) % (6 * 5 * 4 * 3)
                    / (5 * 4 * 3),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3)
                    % (7 * 6 * 5 * 4 * 3)
                    % (6 * 5 * 4 * 3)
                    % (5 * 4 * 3)
                    / (4 * 3),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3)
                    % (7 * 6 * 5 * 4 * 3)
                    % (6 * 5 * 4 * 3)
                    % (5 * 4 * 3)
                    % (4 * 3)
                    / 3,
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3)
                    % (7 * 6 * 5 * 4 * 3)
                    % (6 * 5 * 4 * 3)
                    % (5 * 4 * 3)
                    % (4 * 3)
                    % 3,
            );
            next.check(&result);
            result.push(*next);

            Some(result)
        }
        260650..=623529 => {
            let index = index - 260650;
            let mut result: Vec<usize> = Vec::new();
            let next = Next(index / (8 * 7 * 6 * 5 * 4 * 3 * 2));
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5 * 4 * 3 * 2) / (7 * 6 * 5 * 4 * 3 * 2));
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index % (8 * 7 * 6 * 5 * 4 * 3 * 2) % (7 * 6 * 5 * 4 * 3 * 2) / (6 * 5 * 4 * 3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index % (8 * 7 * 6 * 5 * 4 * 3 * 2) % (7 * 6 * 5 * 4 * 3 * 2) % (6 * 5 * 4 * 3 * 2)
                    / (5 * 4 * 3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    / (4 * 3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    % (4 * 3 * 2)
                    / (3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    % (4 * 3 * 2)
                    % (3 * 2)
                    / 2,
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    % (4 * 3 * 2)
                    % (3 * 2)
                    % 2,
            );
            next.check(&result);
            result.push(*next);

            Some(result)
        }
        623530..=986409 => {
            let index = index - 623530;
            let mut result: Vec<usize> = Vec::new();
            let next = Next(index / (8 * 7 * 6 * 5 * 4 * 3 * 2));
            result.push(*next);
            let mut next = Next(index % (8 * 7 * 6 * 5 * 4 * 3 * 2) / (7 * 6 * 5 * 4 * 3 * 2));
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index % (8 * 7 * 6 * 5 * 4 * 3 * 2) % (7 * 6 * 5 * 4 * 3 * 2) / (6 * 5 * 4 * 3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index % (8 * 7 * 6 * 5 * 4 * 3 * 2) % (7 * 6 * 5 * 4 * 3 * 2) % (6 * 5 * 4 * 3 * 2)
                    / (5 * 4 * 3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    / (4 * 3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    % (4 * 3 * 2)
                    / (3 * 2),
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    % (4 * 3 * 2)
                    % (3 * 2)
                    / 2,
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    % (4 * 3 * 2)
                    % (3 * 2)
                    % 2,
            );
            next.check(&result);
            result.push(*next);
            let mut next = Next(
                index
                    % (8 * 7 * 6 * 5 * 4 * 3 * 2)
                    % (7 * 6 * 5 * 4 * 3 * 2)
                    % (6 * 5 * 4 * 3 * 2)
                    % (5 * 4 * 3 * 2)
                    % (4 * 3 * 2)
                    % (3 * 2)
                    % 2
                    % 1,
            );
            next.check(&result);
            result.push(*next);

            Some(result)
        }
        _ => None,
    }
}

fn takes_3_u8s_and_returns_a_route(u8s: Vec<u8>) -> Result<Vec<usize>, Box<dyn Error>> {
    //检查输入
    if u8s.len() != 3 {
        return Err(Box::new(MyError::new(ErrorKind::LengthError)));
    }

    let mut index = (u8s[0] as u32) << 16;
    index += (u8s[1] as u32) << 8;
    index += u8s[2] as u32;
    loop {
        if index > 986409 {
            index -= 986409;
        } else {
            break;
        }
    }

    // 转换
    let index = index as usize;
    let route: Option<_> = convert(index);

    if let Some(route) = route {
        Ok(route)
    } else {
        // Err(MyError::new(ErrorKind::IndexOutOfMapError).into())
        Ok(vec![3, 1, 5, 7, 8, 0, 4, 2, 6])
    }
}

#[test]
fn test_takes_3_u8s_and_returns_a_route() {
    // for k in 15..16{
    //     for j in 0..255{
    //         for i in 0..255{
    //             match takes_3_u8s_and_returns_a_route(vec![k,j,i]) {
    //                 Ok(route)=>println!("k{}j{}i{} route: {:?}",k,j,i, route),
    //                 Err(_)=>(),
    //             }
    //         }
    //     }
    // }
    match takes_3_u8s_and_returns_a_route(vec![3u8, 3u8, 3u8]) {
        Ok(route) => println!("{:?}", route),
        Err(e) => eprintln!("{e}"),
    }
}

//TODO:测试这个函数
pub fn takes_24_u8s_and_returns_8_routes(u8s: Vec<u8>) -> Result<Vec<Vec<usize>>, Box<dyn Error>> {
    // 检查u8s长度
    if u8s.len() != 24 {
        return Err(MyError::new(ErrorKind::LengthError).into());
    }
    //将要返回的所有routes
    let mut routes = Vec::new();
    let mut iter = u8s.iter();
    for _ in 0..8 {
        let mut u8s = Vec::new();
        for _ in 0..3 {
            u8s.push(*iter.next().unwrap())
        }
        let route = takes_3_u8s_and_returns_a_route(u8s)?;
        routes.push(route);
    }

    Ok(routes)
}

#[test]
fn test_takes_24_u8s_and_returns_8_routes() {
    let u8s = vec![
        0u8, 1u8, 1u8, 0u8, 1u8, 1u8, 0u8, 1u8, 1u8, 0u8, 1u8, 1u8, 0u8, 1u8, 1u8, 0u8, 1u8, 1u8,
        0u8, 1u8, 1u8, 0u8, 1u8, 1u8,
    ];
    match takes_24_u8s_and_returns_8_routes(u8s) {
        Ok(routes) => println!("{:?}", routes),
        Err(e) => eprintln!("{e}"),
    }
}
