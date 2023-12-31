mod parse;

use rayon::prelude::*;
use rgb::RGB8;
use textplots::{Chart, ColorPlot, Shape};

use nom::Finish;
use parse::{Name, Valve};
use std::{
    collections::{HashMap, VecDeque},
    dbg, fs,
    iter::once,
};

#[derive(Debug, Clone)]
struct State {
    lable: Name,
    value: u32,
    time: u32,
    opened: Vec<Name>,
}

impl State {
    fn is_diverge(&self, other: &Self) -> bool {
        for a in self.opened.iter() {
            for b in other.opened.iter() {
                if a == b {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Default)]
struct DistMap {
    data: HashMap<Name, (u32, Vec<Name>)>,
    complete: Vec<State>,
}

impl DistMap {
    fn init(s: &str) -> Self {
        let mut dist = DistMap::default();

        for line in s.lines() {
            let valve = Valve::scrap_valve(line).finish().unwrap().1;

            dist.data.insert(valve.name, (valve.flow, valve.adjecents));
        }
        dist
    }

    fn best_option_greedy(&mut self, state: State) -> Vec<State> {
        let mut queue: VecDeque<(Name, u32)> = VecDeque::new();
        let mut enqueued: Vec<Name> = Vec::new();
        let mut options: Vec<State> = Vec::new();
        queue.push_back((state.lable, state.time));

        while !queue.is_empty() {
            let (name, elapsed) = queue.pop_front().unwrap();

            if elapsed <= 1 {
                break;
            }

            let node = self.data.get(&name).unwrap();

            if node.0 != 0 && !state.opened.contains(&name) {
                options.push(State {
                    lable: name,
                    value: state.value + node.0 * (elapsed - 1),
                    time: elapsed - 1,
                    opened: state.opened.iter().copied().chain(once(name)).collect(),
                });
            }
            node.1.iter().for_each(|v| {
                if !enqueued.contains(&v) {
                    queue.push_back((*v, elapsed - 1));
                    enqueued.push(*v);
                }
            });
        }
        if options.is_empty() {
            self.complete.push(state);
        }
        options
    }

    fn brute_search(&mut self) -> (Vec<(f32, f32)>, Vec<(f32, f32)>) {
        let mut queue = VecDeque::with_capacity(195_000);
        let mut highest: u32 = 0;
        queue.push_back(State {
            lable: Name::from("AA"),
            value: 0,
            time: 26,
            opened: vec![],
        });

        // Variables for statistics
        let mut points: Vec<(f32, f32)> = Vec::new();
        let mut empty_returns: Vec<(f32, f32)> = Vec::new();
        let mut count_x = 0.;
        let mut count_y = 0.;
        #[allow(unused_assignments)]
        let mut count = 0;
        let mut empty = 0.;
        let mut switch = true;

        while !queue.is_empty() {
            count = 0;
            count_x += 1.;
            let state = queue.pop_front().unwrap();
            self.best_option_greedy(state)
                .into_iter()
                .for_each(|sub_state| {
                    count_y += 1.;
                    if sub_state.value > highest {
                        highest = sub_state.value;
                    }
                    count += 1;

                    queue.push_back(sub_state);
                });
            points.push((count_x, count_y));
            if count == 0 {
                empty += 1.;
                switch = true;
            } else if switch {
                empty_returns.push((count_x, empty));
                empty = 0.;
                switch = false
            }
        }
        dbg!(highest);
        (points, empty_returns)
    }

    fn pair_all_possible(&self) {
        let highest = self
            .complete
            .par_iter()
            .enumerate()
            .map(|(i, item1)| {
                self.complete[i..]
                    .par_iter()
                    .map(|item2| {
                        if item1.is_diverge(item2) {
                            item1.value + item2.value
                        } else {
                            0
                        }
                    })
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);

        println!("highest pair is {}", highest);
    }
}

fn main() {
    let start = std::time::Instant::now();
    let file = fs::read_to_string("day16.txt").unwrap();

    let mut map = DistMap::init(&file);

    let (points, empties) = map.brute_search();
    map.pair_all_possible();
    // chart-width, chart-height, dataset-start, dataset-end
    // 280, 90, 0.0, 190_000.0
    Chart::new(280, 80, 0.0, 50_000.0)
        .linecolorplot(
            &Shape::Lines(&points),
            RGB8 {
                r: 0,
                g: 0,
                b: 255_u8,
            },
        )
        .display();
    Chart::new(280, 80, 0.0, 40_000.0)
        .linecolorplot(
            &Shape::Lines(&empties),
            RGB8 {
                r: 255_u8,
                g: 0,
                b: 0,
            },
        )
        .display();

    let end = start.elapsed();
    println!("elapsed time {:?}", end);
}
