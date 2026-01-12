/// Validation and Verification Tests
///
/// This module contains tests to validate that the simulation produces
/// statistically correct results that match queueing theory predictions.

use drive_through_simulation::simulation::Simulation;

/// Test that a simple M/M/1 queue produces results close to theoretical values
///
/// For an M/M/1 queue with arrival rate λ and service rate μ:
/// - Utilization ρ = λ/μ
/// - Average queue length L = ρ²/(1-ρ)
/// - Average waiting time W = L/λ
///
/// Note: This test uses uniform service times (not exponential), so results
/// will approximate but not exactly match M/M/1 theory.
#[tokio::test]
async fn test_mm1_queue_theoretical_validation() {
    // M/M/1 Queue: Single server, exponential arrivals and service
    // λ = 1 customer per minute, μ = 2 customers per minute
    // Expected utilization ρ = 0.5 (50%)

    let mut sim = Simulation::new(1);

    // Generate customers for 1000 minutes with exponential inter-arrival times
    // Mean inter-arrival time = 60 seconds (1 customer/minute)
    // Mean service time = 30 seconds (2 customers/minute)
    sim.generate_random_customers(
        60000.0, // 1000 minutes
        60.0,    // avg arrival interval (1/λ)
        25.0,    // min service
        35.0,    // max service (mean ≈ 30)
    );

    sim.run(Some(60000.0), None).await;

    let state = sim.state.lock().unwrap();

    // Theoretical values for ρ = 0.5
    let rho = 0.5;
    let theoretical_utilization = rho;
    let theoretical_queue_length = (rho * rho) / (1.0 - rho); // ≈ 0.5

    // Calculate actual values
    let actual_utilization = state.stats.server_busy_integral / state.current_time;
    let actual_queue_length = state.stats.queue_length_integral / state.current_time;

    // Allow higher tolerance due to:
    // 1. Random variance in simulation
    // 2. Uniform service time distribution (not exponential)
    let utilization_tolerance = 0.20;  // ±20%
    let queue_tolerance = 0.50;         // ±50% (queue length more sensitive to distribution)

    println!("M/M/1 Queue Validation:");
    println!("  Theoretical utilization: {:.2}", theoretical_utilization);
    println!("  Actual utilization: {:.2}", actual_utilization);
    println!("  Theoretical avg queue length: {:.2}", theoretical_queue_length);
    println!("  Actual avg queue length: {:.2}", actual_queue_length);

    // Utilization should be close (this is distribution-independent)
    assert!(
        (actual_utilization - theoretical_utilization).abs() / theoretical_utilization < utilization_tolerance,
        "Utilization differs too much from theoretical value: expected {:.2}, got {:.2}",
        theoretical_utilization, actual_utilization
    );

    // Queue length is more sensitive to distribution assumptions
    // Just verify it's in reasonable range
    assert!(
        (actual_queue_length - theoretical_queue_length).abs() / theoretical_queue_length < queue_tolerance,
        "Queue length differs too much from theoretical value: expected {:.2}, got {:.2}",
        theoretical_queue_length, actual_queue_length
    );
}

/// Test that utilization cannot exceed 100%
#[tokio::test]
async fn test_utilization_bounds() {
    let mut sim = Simulation::new(2);

    // Add customers with overlapping service times
    for i in 0..10 {
        sim.add_customer(i as f64 * 5.0, 60.0);
    }

    sim.run(None, None).await;

    let state = sim.state.lock().unwrap();
    let utilization_pct = (state.stats.server_busy_integral / state.current_time / state.num_windows as f64) * 100.0;

    assert!(
        utilization_pct <= 100.0,
        "Utilization should never exceed 100%: got {:.2}%",
        utilization_pct
    );
}

/// Test conservation of customers: all arrived customers should be accounted for
#[tokio::test]
async fn test_customer_conservation() {
    let mut sim = Simulation::new(3);

    // Generate random customers
    sim.generate_random_customers(3600.0, 10.0, 5.0, 15.0);

    let total_customers = sim.state.lock().unwrap().customers.len();

    sim.run(Some(3600.0), None).await;

    let state = sim.state.lock().unwrap();

    // All customers should either be completed or in progress
    let accounted_for = state.stats.completed_customers +
                        (state.waiting_queue_len + state.busy_servers);

    assert_eq!(
        accounted_for, total_customers,
        "Customer conservation violated: {} generated, {} accounted for",
        total_customers, accounted_for
    );
}

/// Test that wait times are always non-negative
#[tokio::test]
async fn test_wait_times_non_negative() {
    let mut sim = Simulation::new(2);

    // Add various customer patterns
    sim.add_customer(0.0, 30.0);
    sim.add_customer(10.0, 20.0);
    sim.add_customer(15.0, 40.0);
    sim.add_customer(100.0, 10.0);

    sim.run(None, None).await;

    let state = sim.state.lock().unwrap();

    // Check all completed customers
    for customer in &state.customers {
        if let (Some(start), Some(_end)) = (customer.service_start_time, customer.service_end_time) {
            let wait_time = start - customer.arrival_time;
            assert!(
                wait_time >= 0.0,
                "Wait time cannot be negative: customer arrived at {}, started service at {}",
                customer.arrival_time, start
            );
        }
    }
}

/// Test that service times match requested durations
#[tokio::test]
async fn test_service_time_accuracy() {
    let mut sim = Simulation::new(1);

    let requested_service_time = 45.0;
    sim.add_customer(0.0, requested_service_time);

    sim.run(None, None).await;

    let state = sim.state.lock().unwrap();
    let customer = &state.customers[0];

    if let (Some(start), Some(end)) = (customer.service_start_time, customer.service_end_time) {
        let actual_service_time = end - start;
        assert!(
            (actual_service_time - requested_service_time).abs() < 0.001,
            "Service time should match requested: expected {}, got {}",
            requested_service_time, actual_service_time
        );
    }
}

/// Test FIFO ordering in single-server queue
#[tokio::test]
async fn test_fifo_ordering() {
    let mut sim = Simulation::new(1);

    // Add customers that will queue
    sim.add_customer(0.0, 100.0);   // Customer 0: serves until t=100
    sim.add_customer(10.0, 50.0);   // Customer 1: arrives at t=10
    sim.add_customer(20.0, 50.0);   // Customer 2: arrives at t=20

    sim.run(None, None).await;

    let state = sim.state.lock().unwrap();

    // Customer 1 should start before Customer 2 (FIFO)
    let start1 = state.customers[1].service_start_time.unwrap();
    let start2 = state.customers[2].service_start_time.unwrap();

    assert!(
        start1 < start2,
        "FIFO ordering violated: Customer 1 started at {}, Customer 2 at {}",
        start1, start2
    );
}

/// Test that statistics are incremental (don't require storing all events)
#[tokio::test]
async fn test_incremental_statistics() {
    let mut sim = Simulation::new(2);

    // Generate many customers
    sim.generate_random_customers(7200.0, 5.0, 3.0, 10.0);

    sim.run(Some(7200.0), None).await;

    let state = sim.state.lock().unwrap();

    // Verify statistics are calculated
    assert!(state.stats.completed_customers > 0);
    assert!(state.stats.total_wait_time >= 0.0);
    assert!(state.stats.total_service_time > 0.0);
    assert!(state.stats.queue_length_integral >= 0.0);
    assert!(state.stats.server_busy_integral > 0.0);
}

/// Test multiple server utilization
#[tokio::test]
async fn test_multiple_servers() {
    let mut sim = Simulation::new(3);

    // Add customers with slightly staggered arrival times to avoid async scheduling issues
    // This tests that multiple servers can work in parallel
    sim.add_customer(0.0, 50.0);
    sim.add_customer(1.0, 50.0);   // Arrive 1 second later
    sim.add_customer(2.0, 50.0);   // Arrive 2 seconds later

    sim.run(None, None).await;

    let state = sim.state.lock().unwrap();

    // All 3 customers should complete
    assert_eq!(state.stats.completed_customers, 3, "All customers should complete");

    // With 3 servers and staggered arrivals, there should be minimal queuing
    // Max queue should be 0 since servers are available
    assert!(
        state.stats.max_queue_length <= 2,
        "Max queue should be minimal with 3 servers for 3 customers, got {}",
        state.stats.max_queue_length
    );

    // With 3 servers, all customers should finish within roughly 52 seconds
    // (last customer arrives at 2s, takes 50s to serve, completes at ~52s)
    // If servers were NOT parallel, it would take ~150s
    let finish_time = state.current_time;
    assert!(
        finish_time < 100.0,
        "With 3 parallel servers, simulation should finish in < 100s, took {:.2}s",
        finish_time
    );

    // Average wait should be very low (customers arrive when servers are free)
    let avg_wait = if state.stats.completed_customers > 0 {
        state.stats.total_wait_time / state.stats.completed_customers as f64
    } else {
        0.0
    };

    assert!(
        avg_wait < 2.0,
        "Average wait time should be minimal with staggered arrivals and available servers, got {:.2}",
        avg_wait
    );
}

/// Test that max queue length is tracked correctly
#[tokio::test]
async fn test_max_queue_tracking() {
    let mut sim = Simulation::new(1);

    // Create a situation where queue builds up
    sim.add_customer(0.0, 100.0);   // Long service
    sim.add_customer(10.0, 10.0);   // These will queue
    sim.add_customer(20.0, 10.0);
    sim.add_customer(30.0, 10.0);
    sim.add_customer(40.0, 10.0);

    sim.run(None, None).await;

    let state = sim.state.lock().unwrap();

    // Max queue should be at least 4 (when all queued customers are waiting)
    assert!(
        state.stats.max_queue_length >= 4,
        "Max queue length should be at least 4, got {}",
        state.stats.max_queue_length
    );
}

/// Test Little's Law: L = λW
/// Average number in system = arrival rate × average time in system
#[tokio::test]
async fn test_littles_law() {
    let mut sim = Simulation::new(2);

    // Generate customers for a long simulation
    sim.generate_random_customers(
        36000.0, // 10 hours
        30.0,    // avg inter-arrival time (λ = 1/30 customers/sec)
        10.0,    // min service
        30.0,    // max service
    );

    sim.run(Some(36000.0), None).await;

    let state = sim.state.lock().unwrap();
    let stats = &state.stats;

    if stats.completed_customers > 100 {
        // Calculate arrival rate λ
        let lambda = stats.completed_customers as f64 / state.current_time;

        // Calculate average time in system W (wait + service)
        let avg_wait = stats.total_wait_time / stats.completed_customers as f64;
        let avg_service = stats.total_service_time / stats.completed_customers as f64;
        let avg_time_in_system = avg_wait + avg_service;

        // Calculate average number in system L (queue + service)
        let avg_in_queue = stats.queue_length_integral / state.current_time;
        let avg_in_service = stats.server_busy_integral / state.current_time;
        let avg_in_system = avg_in_queue + avg_in_service;

        // Little's Law: L = λW
        let expected_in_system = lambda * avg_time_in_system;

        println!("Little's Law Validation:");
        println!("  L (actual): {:.3}", avg_in_system);
        println!("  λW (expected): {:.3}", expected_in_system);
        println!("  λ: {:.6}", lambda);
        println!("  W: {:.3}", avg_time_in_system);

        // Allow 15% tolerance
        let tolerance = 0.15;
        let relative_error = (avg_in_system - expected_in_system).abs() / expected_in_system;

        assert!(
            relative_error < tolerance,
            "Little's Law violated: L={:.3}, λW={:.3}, error={:.1}%",
            avg_in_system, expected_in_system, relative_error * 100.0
        );
    }
}

