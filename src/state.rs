use crate::customer::Customer;
use crate::event::EventType;
use crate::output::OutputMessage;
use crate::statistics::Statistics;
use std::fs::File;
use std::io::Write;
use tokio::sync::mpsc;

/// Holds the state of the simulation at any point in time
pub struct SimState {
    pub customers: Vec<Customer>,
    pub waiting_queue_len: usize,
    pub busy_servers: usize,
    pub num_windows: usize,
    pub csv_file: Option<File>,
    pub output_tx: Option<mpsc::UnboundedSender<OutputMessage>>,
    pub current_time: f64,
    pub stats: Statistics,
}

impl SimState {
    /// Updates the time-weighted integrals for queue length and server utilization
    ///
    /// Should be called before any state change to properly track statistics
    pub fn update_integral(&mut self, now: f64) {
        self.stats.update_integrals(now, self.waiting_queue_len, self.busy_servers);
        self.current_time = now;
    }

    /// Records an event in the simulation history and sends it to the output thread
    /// Also streams the event to CSV file if one is open
    pub fn record_history(&mut self, now: f64, event: EventType, cust_id: usize) {
        // Send to output thread for ordered printing
        if let Some(ref tx) = self.output_tx {
            let msg = OutputMessage {
                time: now,
                event,
                cust_id,
                queue_len: self.waiting_queue_len,
                busy_servers: self.busy_servers,
                num_windows: self.num_windows,
            };
            let _ = tx.send(msg);
        }

        // Stream to CSV file if open
        if let Some(ref mut file) = self.csv_file {
            let _ = writeln!(
                file,
                "{:.2},{},{},{},{}",
                now, event, cust_id, self.waiting_queue_len, self.busy_servers
            );
            // Flush CSV file too for real-time streaming
            let _ = file.flush();
        }

        // Track max queue length
        self.stats.update_max_queue(self.waiting_queue_len);

        // Update running statistics based on event type
        if let EventType::ServiceEnd = event
            && cust_id < self.customers.len()
            && let (Some(start), Some(end)) = (
                self.customers[cust_id].service_start_time,
                self.customers[cust_id].service_end_time,
            )
        {
            let arrival = self.customers[cust_id].arrival_time;
            let wait_time = start - arrival;
            let service_time = end - start;

            self.stats.record_completion(wait_time, service_time);
        }
    }

    /// Initialize CSV file for streaming events
    pub fn init_csv(&mut self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        writeln!(file, "Time,Event,CustomerID,QueueLength,BusyServers")?;
        self.csv_file = Some(file);
        Ok(())
    }

    /// Close the CSV file
    pub fn close_csv(&mut self) {
        if let Some(mut file) = self.csv_file.take() {
            let _ = file.flush();
        }
    }
}
