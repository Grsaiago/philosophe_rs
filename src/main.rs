use clap::Parser;
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Parser, Debug)]
struct CliArgs {
    n_philos: u32,
    time_to_die: u32,
    time_to_eat: u32,
    time_to_sleep: u32,
    total_eat: Option<u64>,
}

struct Controller {
    args: CliArgs,
    forks: Vec<Arc<Mutex<u64>>>,
    philos: Vec<Arc<Philosopher>>,
}

struct Philosopher {
    id: u64,
    lfork: Arc<u64>,
    rfork: Arc<u64>,
}

impl Philosopher {
    fn new(id: u64, lfork: Arc<u64>, rfork: Arc<u64>) -> Philosopher {
        Philosopher { id, lfork, rfork }
    }

    fn eat(&self) {
        println!("{}, is done eating", self.id);
    }
}

impl Controller {
    fn new() -> Self {
        let args = CliArgs::parse();
        let n_philos = args.n_philos;
        Controller {
            args,
            forks: Self::create_mutexes(n_philos),
            philos: Self::create_philos(n_philos),
        }
    }

    fn create_mutexes(range: u32) -> Vec<Arc<Mutex<u64>>> {
        let mut mutex_array = Vec::<Arc<Mutex<u64>>>::new();

        for _i in 0..range {
            mutex_array.push(Arc::new(Mutex::new(0)));
        }
        mutex_array
    }

    fn create_philos(range: u32) -> Vec<Arc<Philosopher>> {
        let mut philos: Vec<Arc<Philosopher>> = Vec::new();

        todo!();
        philos
    }
}

fn main() {
    let controller = Controller::new();
    /*
    let threads: Vec<_> = philos
        .into_iter()
        .map(|philo| {
            thread::spawn(move || {
                philo.eat();
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    */
}
