use clap::Parser;
use std::{
    sync::{Arc, Barrier, Mutex, RwLock},
    thread::{self},
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
struct EatControl {
    max: u32,
    count: u32,
}

#[derive(Debug)]
struct Controller {
    n_philos: u32,
    time_to_die: Duration,
    time_to_eat: Duration,
    time_to_sleep: Duration,
    total_eat: Option<u32>,
    forks: Box<[Arc<Mutex<ForkState>>]>,
    execution_state: Arc<RwLock<ExecutionState>>,
    last_eaten: Box<[Arc<Mutex<Option<Instant>>>]>,
    barrier: Arc<Barrier>,
}

#[derive(Debug)]
struct Philosopher {
    id: u32,
    lfork: Arc<Mutex<ForkState>>,
    rfork: Arc<Mutex<ForkState>>,
    time_to_eat: Duration,
    time_to_sleep: Duration,
    eat_control: Option<EatControl>,
    last_eaten: Arc<Mutex<Option<Instant>>>,
    execution_state: Arc<RwLock<ExecutionState>>,
    barrier: Arc<Barrier>,
}

#[derive(Debug)]
struct Observer {
    time_to_die: Duration,
    philos_last_eaten: Box<[Arc<Mutex<Option<Instant>>>]>,
    execution_state: Arc<RwLock<ExecutionState>>,
    barrier: Arc<Barrier>,
}

#[derive(Debug, PartialEq)]
pub enum ExecutionState {
    Stop,
    Continue,
}

#[derive(Debug, PartialEq)]
pub enum ForkState {
    Available,
    Taken,
}

pub enum PhiloAction {
    Eat(u64),
    Sleep(u64),
    Think(u64),
    TakeFork(u64),
}

pub fn print_action(action: PhiloAction) {
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
                    if *val == ForkState::Available {
                        *val = ForkState::Taken;
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
                    if *val == ForkState::Available {
                        *val = ForkState::Taken;
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
                    if *val == ForkState::Available {
                        *val = ForkState::Taken;
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
                    if *val == ForkState::Available {
                        *val = ForkState::Taken;
                        break;
                    }
                }
                if *self.execution_state.read().expect("Error on lock") == ExecutionState::Stop {
                    return ExecutionState::Stop;
                }
            }
        }
        ExecutionState::Continue
    }

    fn eat(&mut self) -> ExecutionState {
        print_action(PhiloAction::Eat(self.id as u64));
        {
            let mut mutex = self.last_eaten.lock().expect("Poisoned mutex");
            *mutex = Some(Instant::now());
        }
        if self.smart_sleep(self.time_to_eat) == ExecutionState::Stop {
            return ExecutionState::Stop;
        }
        if self.id % 2 == 0 {
            let mut llock = self.lfork.lock().expect("Error on locking Fork");
            *llock = ForkState::Available;
            let mut rlock = self.rfork.lock().expect("Error on locking Fork");
            *rlock = ForkState::Available;
        } else {
            let mut rlock = self.rfork.lock().expect("Error on locking Fork");
            *rlock = ForkState::Available;
            let mut llock = self.lfork.lock().expect("Error on locking Fork");
            *llock = ForkState::Available;
        }
        if let Some(ref mut eat_control) = self.eat_control {
            eat_control.count += 1;
            if eat_control.count >= eat_control.max {
                self.last_eaten
                    .lock()
                    .expect("Error on lock last_eaten")
                    .take();
                return ExecutionState::Stop;
            }
        }
        ExecutionState::Continue
    }

    fn sleep(&self) -> ExecutionState {
        print_action(PhiloAction::Sleep(self.id as u64));
        self.smart_sleep(self.time_to_sleep)
    }

    fn smart_sleep(&self, time: Duration) -> ExecutionState {
        let start_time = Instant::now();
        while start_time.elapsed() <= time {
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
        ExecutionState::Continue
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
            last_eaten: (0..n_philos).map(|_| Arc::new(Mutex::new(None))).collect(),
            barrier: Arc::new(Barrier::new(n_philos as usize + 1)),
            //barrier: Arc::new(Barrier::new(n_philos as usize)),
        }
    }

    fn create_mutexes(range: u32) -> Box<[Arc<Mutex<ForkState>>]> {
        let mut mutex_array = Vec::<Arc<Mutex<ForkState>>>::with_capacity(range as usize);

        for _ in 0..range {
            mutex_array.push(Arc::new(Mutex::new(ForkState::Available)));
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
            rfork: if id == self.n_philos {
                Arc::clone(self.forks.first().expect("create_philos Arc::clone()"))
            } else {
                Arc::clone(
                    self.forks
                        .get(id as usize)
                        .expect("create_philos Arc::clone()"),
                )
            },
            time_to_eat: self.time_to_eat,
            time_to_sleep: self.time_to_sleep,
            eat_control: self.total_eat.map(|eat_max| EatControl {
                max: eat_max,
                count: 0,
            }),
            execution_state: self.execution_state.clone(),
            last_eaten: self.last_eaten.get(id as usize - 1).expect("Error").clone(),
            barrier: self.barrier.clone(),
        }
    }
}

fn main() {
    let controller = Controller::new();
    let observer = Observer {
        time_to_die: controller.time_to_die,
        execution_state: controller.execution_state.clone(),
        philos_last_eaten: controller.last_eaten.clone(),
        barrier: controller.barrier.clone(),
    };

    thread::scope(|scope| {
        // spawn observer
        scope.spawn(move || {
            // await on barrier
            observer.barrier.wait();
            // run logic
            for (idx, philo_last_eaten) in observer.philos_last_eaten.iter().enumerate().cycle() {
                if let Some(last_eaten) = *philo_last_eaten.lock().expect("Error on lock") {
                    if Instant::now() - last_eaten > observer.time_to_die {
                        {
                            let mut lock = observer.execution_state.write().expect("Error on lock");
                            *lock = ExecutionState::Stop;
                        }
                        let timestamp = time::SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        println!("[{}] philo {} died", timestamp, idx);
                        return;
                    }
                }
            }
        });
        // create and run philos
        for iteration in 0..controller.n_philos {
            let mut philo = controller.create_philo(iteration + 1);
            scope.spawn(move || {
                println!("Philo {} waiting on barrier", philo.id);
                philo.barrier.wait();
                philo.dine();
            });
        }
    });
}
