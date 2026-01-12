use crate::clock::SimClock;
use crate::customer::Customer;
use crate::duration::{format_duration, format_duration_fixed_width};
use crate::event::EventType;
use crate::output::OutputMessage;
use crate::state::SimState;
use crate::statistics::Statistics;
use rand::Rng;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// A discrete-event simulation of a drive-through service system.
///
/// This simulation uses async/await with a custom SimClock to model
/// customer arrivals, queueing, and service at multiple service windows.
pub struct Simulation {
    clock: Arc<SimClock>,
    state: Arc<Mutex<SimState>>,
}

impl Simulation {
    /// Creates a new simulation with the specified number of service windows
    ///
    /// # Panics
    /// Panics if `num_windows` is 0
    #[must_use]
    pub fn new(num_windows: usize) -> Self {
        assert!(num_windows > 0, "Number of windows must be greater than 0");

        Self {
            clock: Arc::new(SimClock::new()),
            state: Arc::new(Mutex::new(SimState {
                customers: Vec::new(),
                waiting_queue_len: 0,
                busy_servers: 0,
                num_windows,
                csv_file: None,
                output_tx: None,
                current_time: 0.0,
                stats: Statistics::new(),
            })),
        }
    }

    /// Adds a customer to the simulation
    ///
    /// # Panics
    /// Panics if `arrival_time` is negative or `service_duration` is not positive
    pub fn add_customer(&mut self, arrival_time: f64, service_duration: f64) {
        assert!(arrival_time >= 0.0, "Arrival time must be non-negative");
        assert!(service_duration > 0.0, "Service duration must be positive");

        let mut state = self.state.lock().unwrap();
        state.customers.push(Customer {
            arrival_time,
            service_duration,
            service_start_time: None,
            service_end_time: None,
        });
    }

    /// Runs the simulation
    ///
    /// # Arguments
    /// * `max_time` - Optional maximum simulation time. If None, runs until all customers are served.
    /// * `csv_filename` - Optional CSV filename for streaming event history
    pub async fn run(&mut self, max_time: Option<f64>, csv_filename: Option<&str>) {
        // Initialize CSV file if filename provided
        if let Some(filename) = csv_filename
            && let Err(e) = self.state.lock().unwrap().init_csv(filename)
        {
            eprintln!("Warning: Failed to create CSV file {}: {}", filename, e);
        }

        // Create output channel for ordered event printing
        let (output_tx, mut output_rx) = mpsc::unbounded_channel::<OutputMessage>();

        // Set the output channel in state
        self.state.lock().unwrap().output_tx = Some(output_tx);

        // Spawn dedicated output thread for ordered printing
        let output_handle = tokio::spawn(async move {
            while let Some(msg) = output_rx.recv().await {
                println!(
                    "{} {:<15} {:<10} {:<10} {}/{}",
                    format_duration_fixed_width(msg.time),
                    format!("{:?}", msg.event),
                    msg.cust_id,
                    msg.queue_len,
                    msg.busy_servers,
                    msg.num_windows
                );
                let _ = io::stdout().flush();
            }
        });

        println!("Starting simulation (Coroutine-based)...");
        println!(
            "{:>30} {:<15} {:<10} {:<10} BusyServers",
            "Time", "Event", "CustID", "Queue"
        );
        println!(
            "-------------------------------------------------------------------------------------------"
        );
        let _ = io::stdout().flush();

        let (tx, rx) = mpsc::channel::<usize>(1000);
        let shared_rx = Arc::new(tokio::sync::Mutex::new(rx));
        let num_windows = self.state.lock().unwrap().num_windows;

        let local = tokio::task::LocalSet::new();

        for _ in 0..num_windows {
            let state = self.state.clone();
            let clock = self.clock.clone();
            let rx = shared_rx.clone();
            local.spawn_local(async move {
                loop {
                    let cust_id = {
                        let mut rx_lock = rx.lock().await;
                        match rx_lock.recv().await {
                            Some(id) => id,
                            None => break,
                        }
                    };

                    let (duration, _now) = {
                        let mut s = state.lock().unwrap();

                        // Validate customer ID
                        if cust_id >= s.customers.len() {
                            eprintln!("Error: Invalid customer ID {}", cust_id);
                            continue;
                        }

                        let now = clock.now();

                        // Update integral BEFORE changing state (captures old state correctly)
                        s.update_integral(now);

                        // Now change state
                        s.busy_servers += 1;

                        // Prevent underflow: only decrement if queue has customers
                        if s.waiting_queue_len > 0 {
                            s.waiting_queue_len -= 1;
                        } else {
                            eprintln!("Warning: Queue underflow prevented at T={}", clock.now());
                        }

                        s.customers[cust_id].service_start_time = Some(now);
                        s.record_history(now, EventType::ServiceStart, cust_id);
                        (s.customers[cust_id].service_duration, now)
                    };

                    clock.sleep(duration).await;

                    {
                        let mut s = state.lock().unwrap();
                        let now = clock.now();

                        // Update integral BEFORE changing state
                        s.update_integral(now);

                        // Now change state
                        s.busy_servers -= 1;

                        s.customers[cust_id].service_end_time = Some(now);
                        s.record_history(now, EventType::ServiceEnd, cust_id);
                    }
                }
            });
        }

        let arrival_state = self.state.clone();
        let arrival_clock = self.clock.clone();
        local.spawn_local(async move {
            let customers_len = arrival_state.lock().unwrap().customers.len();
            for i in 0..customers_len {
                let arrival_time = arrival_state.lock().unwrap().customers[i].arrival_time;
                if max_time.is_some_and(|limit| arrival_time > limit) {
                    break;
                }
                arrival_clock.sleep_until(arrival_time).await;

                // First, send customer to queue to guarantee FIFO order
                // This ensures the channel receives customers in arrival order
                if tx.send(i).await.is_err() {
                    eprintln!(
                        "Warning: All servers shut down prematurely at T={}",
                        arrival_time
                    );
                    break;
                }

                // Then update state and record arrival
                {
                    let mut s = arrival_state.lock().unwrap();
                    s.update_integral(arrival_time);
                    s.waiting_queue_len += 1;
                    s.record_history(arrival_time, EventType::Arrival, i);
                }
            }
            drop(tx);
        });

        local
            .run_until(async {
                let mut no_advance_count = 0;
                const MAX_NO_ADVANCE: usize = 100;

                loop {
                    tokio::task::yield_now().await;

                    if max_time.is_some_and(|limit| self.clock.now() >= limit) {
                        break;
                    }

                    if !self.clock.advance() {
                        // No events to advance - check if we're deadlocked or truly done
                        no_advance_count += 1;

                        if no_advance_count > MAX_NO_ADVANCE {
                            // Potential deadlock - check if there are customers still in system
                            let state = self.state.lock().unwrap();
                            let customers_in_system = state.waiting_queue_len + state.busy_servers;

                            if customers_in_system > 0 {
                                eprintln!(
                                    "Warning: Deadlock detected with {} customers still in system",
                                    customers_in_system
                                );
                            }
                            break;
                        }
                    } else {
                        // Successfully advanced, reset counter
                        no_advance_count = 0;
                    }
                }
            })
            .await;

        // Finalize state tracking
        {
            let mut s = self.state.lock().unwrap();
            let final_time = if let Some(limit) = max_time {
                limit
            } else {
                // Natural completion - use clock time
                self.clock.now()
            };

            // Update integrals for final time period if needed
            if s.current_time < final_time {
                let queue_len = s.waiting_queue_len;
                let busy_servers = s.busy_servers;
                s.stats
                    .update_integrals(final_time, queue_len, busy_servers);
                s.current_time = final_time;
            }

            // Close CSV file first
            s.close_csv();

            // Close output channel to signal output thread to finish
            s.output_tx = None;
        }

        // Wait for output thread to finish printing all messages
        let _ = output_handle.await;

        println!(
            "-------------------------------------------------------------------------------------------"
        );
        println!(
            "Simulation finished at T={}",
            format_duration(self.state.lock().unwrap().current_time)
        );
    }

    /// Prints detailed statistics about the simulation results
    pub fn print_statistics(&self) {
        let state = self.state.lock().unwrap();
        state
            .stats
            .print_report(state.current_time, state.customers.len(), state.num_windows);
    }

    /// Generates random customers using exponential inter-arrival times
    ///
    /// # Arguments
    /// * `max_time` - Maximum simulation time
    /// * `avg_arrival_interval` - Average time between customer arrivals
    /// * `min_service` - Minimum service time
    /// * `max_service` - Maximum service time
    ///
    /// # Panics
    /// Panics if any arguments are invalid (e.g., negative or zero values)
    pub fn generate_random_customers(
        &mut self,
        max_time: f64,
        avg_arrival_interval: f64,
        min_service: f64,
        max_service: f64,
    ) {
        assert!(max_time > 0.0, "Max time must be positive");
        assert!(
            avg_arrival_interval > 0.0,
            "Average arrival interval must be positive"
        );
        assert!(min_service > 0.0, "Minimum service time must be positive");
        assert!(
            max_service >= min_service,
            "Maximum service time must be >= minimum service time"
        );

        let mut rng = rand::rng();
        let mut current_arrival = 0.0;

        loop {
            let u: f64 = 1.0 - rng.random::<f64>();
            let interval = -u.ln() * avg_arrival_interval;
            current_arrival += interval;

            if current_arrival > max_time {
                break;
            }

            let service = rng.random_range(min_service..=max_service);
            self.add_customer(current_arrival, service);
        }
    }
}
