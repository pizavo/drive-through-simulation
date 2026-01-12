use crate::duration::format_duration;

/// Tracks running statistics for the simulation
#[derive(Debug)]
pub struct Statistics {
    // Running totals
    pub total_wait_time: f64,
    pub total_service_time: f64,
    pub completed_customers: usize,

    // Time-weighted integrals
    pub queue_length_integral: f64,
    pub server_busy_integral: f64,

    // Peak values
    pub max_wait_time: f64,
    pub max_queue_length: usize,

    // Tracking state
    pub last_event_time: f64,
}

impl Statistics {
    /// Creates a new Statistics tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_wait_time: 0.0,
            total_service_time: 0.0,
            completed_customers: 0,
            queue_length_integral: 0.0,
            server_busy_integral: 0.0,
            max_wait_time: 0.0,
            max_queue_length: 0,
            last_event_time: 0.0,
        }
    }

    /// Updates the time-weighted integrals
    pub fn update_integrals(&mut self, now: f64, queue_len: usize, busy_servers: usize) {
        let time_passed = now - self.last_event_time;
        if time_passed > 0.0 {
            self.queue_length_integral += time_passed * queue_len as f64;
            self.server_busy_integral += time_passed * busy_servers as f64;
            self.last_event_time = now;
        }
    }

    /// Records a completed customer's statistics
    pub fn record_completion(&mut self, wait_time: f64, service_time: f64) {
        self.total_wait_time += wait_time;
        self.total_service_time += service_time;
        self.completed_customers += 1;

        if wait_time > self.max_wait_time {
            self.max_wait_time = wait_time;
        }
    }

    /// Updates the maximum queue length if current exceeds it
    pub fn update_max_queue(&mut self, current_queue_len: usize) {
        if current_queue_len > self.max_queue_length {
            self.max_queue_length = current_queue_len;
        }
    }

    /// Prints comprehensive statistics report
    pub fn print_report(&self, current_time: f64, total_customers: usize, num_windows: usize) {
        println!("\nSimulation Statistics:");
        println!("-----------------------------------------------");
        println!("Total customers processed: {}", total_customers);
        println!("Customers completed: {}", self.completed_customers);

        if self.completed_customers > 0 {
            let avg_wait = self.total_wait_time / self.completed_customers as f64;
            let avg_service = self.total_service_time / self.completed_customers as f64;

            println!(
                "Average waiting time per customer: {}",
                format_duration(avg_wait)
            );
            println!(
                "Maximum waiting time: {}",
                format_duration(self.max_wait_time)
            );
            println!(
                "Average service time per customer: {}",
                format_duration(avg_service)
            );
        }

        if current_time > 0.0 {
            let avg_queue_length = self.queue_length_integral / current_time;
            println!(
                "Average queue length (time-weighted): {:.0} customers",
                avg_queue_length.round()
            );
            println!(
                "Maximum queue length: {} customers",
                self.max_queue_length
            );

            let avg_busy_servers = self.server_busy_integral / current_time;
            println!(
                "Average servers busy (time-weighted): {:.0} of {} windows",
                avg_busy_servers.round(), num_windows
            );
            let utilization_pct = (avg_busy_servers / num_windows as f64) * 100.0;
            println!("Server utilization: {:.2}%", utilization_pct);

            // Calculate throughput (customers per hour)
            let hours = current_time / 3600.0;
            if hours > 0.0 {
                let throughput = self.completed_customers as f64 / hours;
                println!("Throughput: {:.2} customers/hour", throughput);
            }
        }

        // Show in-progress customers if any
        let in_progress = total_customers - self.completed_customers;
        if in_progress > 0 {
            println!("\nNote: {} customers still in system (waiting or being served)", in_progress);
        }
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_new() {
        let stats = Statistics::new();
        assert_eq!(stats.total_wait_time, 0.0);
        assert_eq!(stats.total_service_time, 0.0);
        assert_eq!(stats.completed_customers, 0);
        assert_eq!(stats.max_wait_time, 0.0);
        assert_eq!(stats.max_queue_length, 0);
    }

    #[test]
    fn test_record_completion() {
        let mut stats = Statistics::new();

        stats.record_completion(10.0, 20.0);
        assert_eq!(stats.completed_customers, 1);
        assert_eq!(stats.total_wait_time, 10.0);
        assert_eq!(stats.total_service_time, 20.0);
        assert_eq!(stats.max_wait_time, 10.0);

        stats.record_completion(15.0, 25.0);
        assert_eq!(stats.completed_customers, 2);
        assert_eq!(stats.total_wait_time, 25.0);
        assert_eq!(stats.total_service_time, 45.0);
        assert_eq!(stats.max_wait_time, 15.0);
    }

    #[test]
    fn test_update_integrals() {
        let mut stats = Statistics::new();

        // Scenario: queue starts with 2 customers, 1 server busy at t=0
        // At t=10, state changes to 3 in queue, 2 servers busy
        // At t=20, state changes to 1 in queue, 1 server busy

        // Call update_integrals with the state that existed from t=0 to t=10
        stats.update_integrals(10.0, 2, 1);
        assert_eq!(stats.last_event_time, 10.0);
        assert_eq!(stats.queue_length_integral, 20.0); // 2 customers * 10 seconds
        assert_eq!(stats.server_busy_integral, 10.0);  // 1 server * 10 seconds

        // Call update_integrals with the state that existed from t=10 to t=20
        stats.update_integrals(20.0, 3, 2);
        assert_eq!(stats.queue_length_integral, 50.0); // 20 + (3 * 10)
        assert_eq!(stats.server_busy_integral, 30.0);  // 10 + (2 * 10)

        // Call update_integrals with the state that existed from t=20 to t=30
        stats.update_integrals(30.0, 1, 1);
        assert_eq!(stats.queue_length_integral, 60.0); // 50 + (1 * 10)
        assert_eq!(stats.server_busy_integral, 40.0);  // 30 + (1 * 10)
    }

    #[test]
    fn test_update_max_queue() {
        let mut stats = Statistics::new();

        stats.update_max_queue(5);
        assert_eq!(stats.max_queue_length, 5);

        stats.update_max_queue(3);
        assert_eq!(stats.max_queue_length, 5); // Should stay at max

        stats.update_max_queue(10);
        assert_eq!(stats.max_queue_length, 10); // New max
    }

    #[test]
    fn test_averages() {
        let mut stats = Statistics::new();

        stats.record_completion(5.0, 15.0);
        stats.record_completion(10.0, 20.0);
        stats.record_completion(15.0, 25.0);

        let avg_wait = stats.total_wait_time / stats.completed_customers as f64;
        let avg_service = stats.total_service_time / stats.completed_customers as f64;

        assert_eq!(avg_wait, 10.0); // (5 + 10 + 15) / 3
        assert_eq!(avg_service, 20.0); // (15 + 20 + 25) / 3
    }

    #[test]
    fn test_time_weighted_averages() {
        let mut stats = Statistics::new();

        // Simulate: 0 in queue for 10s, then 5 in queue for 20s
        stats.update_integrals(10.0, 0, 0);
        stats.update_integrals(30.0, 5, 1);

        let total_time = 30.0;
        let avg_queue = stats.queue_length_integral / total_time;

        // Should be (0*10 + 5*20) / 30 = 100/30 = 3.33
        assert!((avg_queue - 3.333).abs() < 0.01);
    }
}

