# The philosophers' dinner but make it 🦀rusty🦀
This project is a rust concurrent implementation of Dijkstra's Dinning Philosophers Problem [Dining Philosophers Problem](https://en.wikipedia.org/wiki/Dining_philosophers_problem).

This project uses:
- [Clap](https://docs.rs/clap/latest/clap/) to parse the arguments from the CLI into a config struct.
- [std::sync::Barrier](https://doc.rust-lang.org/stable/std/sync/struct.Barrier.html) to sincronize the start of computation.
- [std::sync::RwLock](https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html) to sync areas with lots of reads and just a couple of writes.
- [std::sync::Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) to protect critical areas.
- [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) smart pointer to atomic ref count.
<!---
[✨Tokio✨](https://tokio.rs/) as an async runtime.
!--->

## Rules
The program takes the following arguments:

`./philo <number_of_philosophers> <time_to_die> <time_to_eat> <time_to_sleep> [number_of_times_each_philosopher_must_eat]`

- `number_of_philosophers`: The number of philosophers (and also the number of forks).
- `time_to_die` in ms: If a philosopher didn’t start eating time_to_die
milliseconds since the beginning of their last meal or the beginning of the simulation, they die.
- `time_to_eat` in ms: The time it takes for a philosopher to eat.
- `time_to_sleep` in ms: The time a philosopher will spend sleeping.
- `number_of_times_each_philosopher_must_eat` (optional argument): If all philosophers have eaten at least `number_of_times_each_philosopher_must_eat` times, the simulation stops. If not specified, the simulation stops when a philosopher dies.

## Output
All philosophers log the following actions with the following format:
- Fork taken
- Started to eat
- Started to sleep
- Started to think

```sh
<timestamp in unix epoch> <philosopher id> <action>
```
Messages are always logged in sequence.

## Installation
For a quick dry run, go for:
```sh
cargo r -- <params>
```
