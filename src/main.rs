use clap::Parser;
use std::{
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::{self, Duration, Instant, UNIX_EPOCH},
};

#[derive(Parser, Debug)]
struct CliArgs {
    n_philos: u32,
    time_to_die: u32,
    time_to_eat: u32,
    time_to_sleep: u32,
    total_eat: Option<u32>,
}

#[derive(Debug)]
struct Controller {
    n_philos: u32,
    time_to_die: Duration,
    time_to_eat: Duration,
    time_to_sleep: Duration,
    total_eat: Option<u32>,
    forks: Box<[Arc<Mutex<bool>>]>,
    execution_state: Arc<RwLock<ExecutionState>>,
    last_eaten: Box<[Arc<Mutex<Option<Instant>>>]>,
    threads: Vec<JoinHandle<()>>,
}

#[derive(Debug)]
struct Philosopher {
    id: u32,
    lfork: Arc<Mutex<bool>>,
    rfork: Arc<Mutex<bool>>,
    time_to_die: Duration,
    time_to_eat: Duration,
    time_to_sleep: Duration,
    last_eaten: Arc<Mutex<Option<Instant>>>,
    execution_state: Arc<RwLock<ExecutionState>>,
}

#[derive(Debug, PartialEq)]
pub enum ExecutionState {
    Stop,
    Continue,
}

pub enum PhiloAction {
    Eat(u64),
    Sleep(u64),
    Think(u64),
    TakeFork(u64),
}

pub fn print_action(action: PhiloAction) -> () {
    let timestamp = time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    match action {
        PhiloAction::Eat(var) => {
            println!("[{}] {} is eating", timestamp, var);
        }
        PhiloAction::Sleep(var) => {
            println!("[{}] {} is sleeping", timestamp, var);
        }
        PhiloAction::Think(var) => {
            println!("[{}] {} is thinking", timestamp, var);
        }
        PhiloAction::TakeFork(var) => {
            println!("[{}] {} has taken a fork", timestamp, var);
        }
    }
}

impl Philosopher {
    fn dine(&mut self) {
        loop {
            if self.think() == ExecutionState::Stop {
                return;
            }
            if self.eat() == ExecutionState::Stop {
                return;
            }
            if self.sleep() == ExecutionState::Stop {
                return;
            }
        }
    }

    fn think(&self) -> ExecutionState {
        // try get forks
        print_action(PhiloAction::Think(self.id as u64));
        if self.id % 2 == 0 {
            loop {
                {
                    let mut val = self.lfork.lock().expect("Error on lock");
                    if *val == false {
                        *val = true;
                        break;
                    }
                }
                if *self.execution_state.read().expect("Error on lock") == ExecutionState::Stop {
                    return ExecutionState::Stop;
                }
            }
            loop {
                {
                    let mut val = self.rfork.lock().expect("Error on lock");
                    if *val == false {
                        *val = true;
                        break;
                    }
                }
                if *self.execution_state.read().expect("Error on lock") == ExecutionState::Stop {
                    return ExecutionState::Stop;
                }
            }
        } else {
            loop {
                {
                    let mut val = self.rfork.lock().expect("Error on lock");
                    if *val == false {
                        *val = true;
                        break;
                    }
                }
                if *self.execution_state.read().expect("Error on lock") == ExecutionState::Stop {
                    return ExecutionState::Stop;
                }
            }
            loop {
                {
                    let mut val = self.lfork.lock().expect("Error on lock");
                    if *val == false {
                        *val = true;
                        break;
                    }
                }
                if *self.execution_state.read().expect("Error on lock") == ExecutionState::Stop {
                    return ExecutionState::Stop;
                }
            }
        }
        return ExecutionState::Continue;
    }

    fn eat(&mut self) -> ExecutionState {
        print_action(PhiloAction::Eat(self.id as u64));
        {
            let mut mutex = self.last_eaten.lock().expect("Poisoned mutex");
            *mutex = Some(Instant::now());
        }
        if self.smart_sleep(self.time_to_sleep) == ExecutionState::Stop {
            return ExecutionState::Stop;
        }
        if self.id % 2 == 0 {
            let mut llock = self.lfork.lock().expect("Error on locking Fork");
            *llock = false;
            let mut rlock = self.rfork.lock().expect("Error on locking Fork");
            *rlock = false;
        } else {
            let mut rlock = self.rfork.lock().expect("Error on locking Fork");
            *rlock = false;
            let mut llock = self.lfork.lock().expect("Error on locking Fork");
            *llock = false;
        }
        return ExecutionState::Continue;
    }

    fn sleep(&self) -> ExecutionState {
        print_action(PhiloAction::Sleep(self.id as u64));
        return self.smart_sleep(self.time_to_sleep);
    }

    fn smart_sleep(&self, time: Duration) -> ExecutionState {
        let start_time = Instant::now();
        while start_time.elapsed() >= time {
            if *self
                .execution_state
                .read()
                .expect("Error on smart sleep lock")
                == ExecutionState::Stop
            {
                return ExecutionState::Stop;
            }
            thread::sleep(time / 10);
        }
        return ExecutionState::Continue;
    }
}

impl Controller {
    fn new() -> Self {
        let args = CliArgs::parse();
        let n_philos = args.n_philos;
        let forks = Self::create_mutexes(n_philos);
        Controller {
            n_philos: args.n_philos,
            time_to_die: Duration::from_millis(args.time_to_die as u64),
            time_to_eat: Duration::from_millis(args.time_to_eat as u64),
            time_to_sleep: Duration::from_millis(args.time_to_sleep as u64),
            total_eat: args.total_eat,
            forks,
            execution_state: Arc::new(RwLock::new(ExecutionState::Continue)),
            threads: vec![],
            last_eaten: (0..n_philos).map(|_| Arc::new(Mutex::new(None))).collect(),
        }
    }

    fn create_mutexes(range: u32) -> Box<[Arc<Mutex<bool>>]> {
        let mut mutex_array = Vec::<Arc<Mutex<bool>>>::with_capacity(range as usize);

        for _ in 0..range {
            mutex_array.push(Arc::new(Mutex::new(false)));
        }
        mutex_array.into_boxed_slice()
    }

    fn create_philo(&self, id: u32) -> Philosopher {
        Philosopher {
            id,
            lfork: Arc::clone(
                self.forks
                    .get(id as usize - 1)
                    .expect("create_philos Arc::clone()"),
            ),
            rfork: if id as u32 == self.n_philos {
                Arc::clone(self.forks.get(0).expect("create_philos Arc::clone()"))
            } else {
                Arc::clone(
                    self.forks
                        .get(id as usize)
                        .expect("create_philos Arc::clone()"),
                )
            },
            time_to_die: self.time_to_die,
            time_to_eat: self.time_to_eat,
            time_to_sleep: self.time_to_sleep,
            execution_state: self.execution_state.clone(),
            last_eaten: Arc::clone(&self.last_eaten.get(id as usize - 1).expect("Error")),
        }
    }
}

fn main() {
    let mut controller = Controller::new();

    let mut threads: Vec<_> = (0..controller.n_philos)
        .into_iter()
        .map(|index| {
            let mut philo = controller.create_philo(index + 1);
            Some(thread::spawn(move || {
                philo.dine();
            }))
        })
        .collect();
    for handle in threads.iter_mut() {
        if let Some(_) = handle.take().map(|var| var.join()) {}
    }
    /*
    for handle in threads.iter_mut() {
        if let Some(_) = handle.take().map(|var| var.join()) {
            println!("A thread terminou");
        }
    }
    */
}
