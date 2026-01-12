# Drive-Through Simulation

A discrete-event simulation of a drive-through service system implemented in Rust using async/await and a custom simulation clock.

## Overview

This simulation models customer arrivals, queueing, and service at a drive-through facility with multiple service windows. It uses coroutine-based scheduling to simulate concurrent customer processing and provides detailed statistics about wait times, queue lengths, and server utilization.

## Features

- 🚗 **Multiple Service Windows** - Simulate drive-throughs with configurable number of service windows
- ⏰ **Custom Simulation Clock** - Discrete-event simulation using a custom async clock implementation
- 📊 **Incremental Statistics** - Real-time calculation of averages without storing all events in memory
- 🎲 **Dual Simulation Modes**:
  - **Fixed Mode** - Use predefined customer arrival and service times
  - **Random Mode** - Generate customers using exponential arrival distribution
- 💾 **Streaming CSV Export** - Events written to CSV file as they occur (no memory buffering)
- ⚙️ **YAML Configuration** - Human-readable configuration with support for duration formats like "1m 30s"
- 🔍 **Ordered Real-Time Output** - Dedicated output thread ensures chronologically ordered event display
- 📐 **Fixed-Width Formatting** - Column-aligned output with compact units (y, m, d, h, min, s, ms) and zero-padding
- 🚀 **Memory Efficient** - Can handle very long simulations without storing event history

## Installation

### Prerequisites

- Rust 1.75+ (with Edition 2024 support)
- Cargo

### Build from Source

```bash
cargo build --release
```

The optimized executable will be created at:
- **Windows:** `target\release\drive-through-simulation.exe`
- **Linux/Mac:** `target/release/drive-through-simulation`

## Building an Executable

### Release Build (Recommended)

To create an optimized, standalone executable:

```bash
cargo build --release
```

This creates an optimized binary at `target\release\drive-through-simulation.exe` (Windows) or `target/release/drive-through-simulation` (Linux/Mac).

**Benefits of release builds:**
- ✅ Optimized for speed (10-100x faster than debug builds)
- ✅ Smaller binary size
- ✅ Production-ready performance

### Debug Build

For development and debugging:

```bash
cargo build
```

Creates an unoptimized binary at `target\debug\drive-through-simulation.exe` with debug symbols.

### Running the Executable

After building in release mode, you can run the executable directly:

**Windows:**
```powershell
# With default config.yaml
.\target\release\drive-through-simulation.exe

# With custom config file
.\target\release\drive-through-simulation.exe --config my-config.yaml
```

**Linux/Mac:**
```bash
# With default config.yaml
./target/release/drive-through-simulation

# With custom config file
./target/release/drive-through-simulation --config my-config.yaml
```

**💡 Tip:** You can place the executable anywhere and specify the full path to your config file:
```powershell
C:\MySimulations\drive-through-simulation.exe --config C:\MySimulations\configs\scenario1.yaml
```

### Distributing the Executable

To distribute your executable:

1. **Copy the executable** from `target\release\`
2. **Include one or more configuration files** (e.g., `config.yaml`)
3. **Dependencies:** The Rust executable is statically linked (on Windows, it may require the Visual C++ runtime)

Example distribution structure:
```
my-simulation/
├── drive-through-simulation.exe
├── config.yaml                    # Default config
├── scenarios/                     # Optional: multiple configs
│   ├── busy-hour.yaml
│   ├── slow-day.yaml
│   └── stress-test.yaml
└── README.txt
```

Users can then run different scenarios:
```powershell
# Default scenario
.\drive-through-simulation.exe

# Specific scenario
.\drive-through-simulation.exe --config scenarios\busy-hour.yaml
```

## Usage

### Basic Usage

Run with the default configuration file (`config.yaml`):

```bash
cargo run --release
```

Or run the executable directly:

```powershell
# Windows
.\target\release\drive-through-simulation.exe

# Linux/Mac
./target/release/drive-through-simulation
```

### Using a Custom Configuration File

You can specify a different configuration file using the `--config` or `-c` option:

```powershell
# Using cargo
cargo run --release -- --config my-custom-config.yaml

# Using the executable directly
.\target\release\drive-through-simulation.exe --config my-custom-config.yaml

# Short form
.\target\release\drive-through-simulation.exe -c my-custom-config.yaml
```

### Command-Line Options

```bash
drive-through-simulation [OPTIONS]

Options:
  -c, --config <CONFIG>  Path to the configuration file [default: config.yaml]
  -h, --help             Print help information
  -V, --version          Print version information
```

**Examples:**

```powershell
# Use default config.yaml
.\drive-through-simulation.exe

# Use a custom config file
.\drive-through-simulation.exe --config scenarios\busy-morning.yaml

# Show help
.\drive-through-simulation.exe --help

# Show version
.\drive-through-simulation.exe --version
```

**⚡ Real-Time Output:**

The simulation flushes stdout after each event, ensuring true real-time console output. However, note:

- ✅ **Direct execution:** Events appear immediately when running the executable directly
- ⚠️ **PowerShell piping:** If you pipe output through PowerShell commands (e.g., `| Select-Object`), PowerShell itself may buffer the output
- 💡 **Tip:** For best real-time experience, run the executable directly without piping

```powershell
# Real-time output (recommended)
.\drive-through-simulation.exe

# May appear buffered due to PowerShell piping
.\drive-through-simulation.exe | Out-File log.txt
```

The simulation will load configuration from `config.yaml` by default, or from the file specified with `--config`.

### Configuration

Create or edit `config.yaml`:

```yaml
fixed_simulation:
  enabled: true
  num_windows: 1
  customers:
    - { arrival: 0, service: "30s" }
    - { arrival: "25s", service: 120 }
    - { arrival: 50, service: "1min 2s" }
  history_file: "history_fixed.csv"

random_simulation:
  enabled: true
  num_windows: 2
  avg_arrival_interval: 40
  min_service_time: 10
  max_service_time: "5min"
  max_simulation_time: "1h"
  history_file: "history_random.csv"
```

**⚠️ Important:** At least one simulation must be enabled. You have three options:
- ✅ **Both simulations** - Set both `enabled: true` (runs both sequentially)
- ✅ **Fixed only** - Set `fixed_simulation.enabled: true` and `random_simulation.enabled: false`
- ✅ **Random only** - Set `fixed_simulation.enabled: false` and `random_simulation.enabled: true`
- ❌ **Neither** - Setting both to `false` will result in an error

#### Duration Formats

Durations can be specified as:
- **Seconds** (numeric): `30`, `120`, `3600`
- **Human-readable**: `"30s"`, `"1m 30s"`, `"2min"`, `"1h"`, `"5min"`

#### Configuration Parameters

**Fixed Simulation:**
- `enabled` - Enable/disable fixed simulation
- `num_windows` - Number of service windows
- `customers` - List of customers with arrival and service times
- `history_file` - CSV output file path

**Random Simulation:**
- `enabled` - Enable/disable random simulation
- `num_windows` - Number of service windows
- `avg_arrival_interval` - Average time between customer arrivals (exponential distribution)
- `min_service_time` - Minimum service time
- `max_service_time` - Maximum service time
- `max_simulation_time` - Total simulation duration
- `history_file` - CSV output file path

### Environment Variables

Override configuration using environment variables with the `APP__` prefix:

```bash
APP__RANDOM_SIMULATION__NUM_WINDOWS=3 cargo run
APP__RANDOM_SIMULATION__MAX_SIMULATION_TIME=7200 cargo run
```

## Example Output

```
=== Drive-Through Simulation System ===
Enabled simulations:
  ✓ Fixed simulation
  ✓ Random simulation

=== Drive-Through Simulation (Fixed Data from Config) ===
Starting simulation (Coroutine-based)...
                          Time Event           CustID     Queue      BusyServers
-------------------------------------------------------------------------------------------
                          0ms Arrival         0          1          0/1
                          0ms ServiceStart    0          0          1/1
                         25s  Arrival         1          1          1/1
                         30s  ServiceEnd      0          1          0/1
                         30s  ServiceStart    1          0          1/1
                         50s  Arrival         2          1          1/1
                      2m 30s  ServiceEnd      1          1          0/1
                      2m 30s  ServiceStart    2          0          1/1
                     3m 32s   ServiceEnd      2          0          0/1
-------------------------------------------------------------------------------------------
Simulation finished at T=3m 32s

Simulation Statistics:
-----------------------------------------------
Total customers processed: 3
Customers completed: 3
Average waiting time per customer: 35s
Average service time per customer: 1m 10s 667ms
Average queue length (time-weighted): 0 customers
Average servers busy (time-weighted): 1 of 1 windows
Server utilization: 100.00%
```

**Output Features:**
- ✅ **Chronologically ordered** - Events from multiple async tasks are ordered by simulation time
- ✅ **Fixed-width columns** - Time column right-aligned in 30-character field for consistent alignment
- ✅ **Compact units** - Short format (y, m, d, h, min, s, ms) saves space while remaining readable
- ✅ **Zero-padded** - Consistent width (e.g., "09s" not "9s" when larger units present)
- ✅ **Scales seamlessly** - Format handles milliseconds to years without breaking alignment
- ✅ **Real-time streaming** - Events appear as they occur, not buffered

**Time Format Examples:**
```
                          9ms  ← Only milliseconds
                         19s   ← Only seconds
                   2min 30s    ← Minutes and seconds
              1h 05min 09s     ← Hours, minutes, seconds (zero-padded)
         3d 12h 30min 45s      ← Days, hours, minutes, seconds
    2m 15d 08h 30min 15s 500ms ← Months, days, hours, minutes, seconds, milliseconds
0001y 06m 15d 12h 00min 00s    ← Years (4 digits), all components zero-padded
```

## Project Structure

```
drive-through-simulation/
├── src/
│   ├── main.rs              # Application entry point
│   ├── simulation.rs        # Main simulation logic
│   ├── clock.rs             # Custom async simulation clock
│   ├── state.rs             # Simulation state management
│   ├── customer.rs          # Customer data structure
│   ├── event.rs             # Event type definitions
│   ├── history.rs           # Event history tracking
│   ├── duration.rs          # Duration parsing and formatting
│   └── config/              # Configuration modules
│       ├── mod.rs           # Config loading
│       ├── fixed.rs         # Fixed simulation config
│       ├── random.rs        # Random simulation config
│       └── customer.rs      # Customer config
├── Cargo.toml
├── config.yaml
└── README.md
```

## How It Works

### Discrete-Event Simulation

The simulation uses a custom `SimClock` that manages virtual time:

1. **Events are scheduled** - Customers arrive, service starts/ends
2. **Clock advances** - Time jumps to the next scheduled event
3. **Tasks wake up** - Async tasks waiting for that time are resumed
4. **State updates** - Queue lengths, server states, and statistics are updated

### Async Architecture

- **Server Tasks** - Each service window runs as an async task, waiting for customers from a shared channel
- **Arrival Task** - Schedules customer arrivals and adds them to the queue
- **Main Loop** - Advances the simulation clock until completion

### Statistics Tracking

The simulation calculates comprehensive statistics:

**Time-Weighted Averages:**
- **Average Queue Length** - ∫(queue_length × dt) / total_time
  - The average number of customers waiting in queue over time
  - Can be fractional (e.g., 2.3 customers means sometimes 2, sometimes 3)
- **Average Servers Busy** - ∫(busy_servers × dt) / total_time
  - The average number of servers actively serving customers
  - Used to calculate server utilization percentage

**Per-Customer Averages:**
- **Average Wait Time** - Σ(wait_time) / num_customers
- **Average Service Time** - Σ(service_time) / num_customers

All time-weighted statistics properly account for the duration each state was active, providing accurate long-term averages.

## Dependencies

- **tokio** - Async runtime
- **rand** - Random number generation
- **serde** - Serialization/deserialization
- **config** - Configuration management
- **humantime** - Human-readable duration parsing

## Testing

Run tests with:

```bash
cargo test
```

Run with Clippy for additional checks:

```bash
cargo clippy
```

## CSV Output

The simulation **streams** event history to CSV files in real-time as events occur (not buffered in memory).

**CSV Format:**

- `Time` - Simulation time when event occurred
- `Event` - Event type (Arrival, ServiceStart, ServiceEnd)
- `CustomerID` - Customer identifier
- `QueueLength` - Number of customers waiting
- `BusyServers` - Number of servers currently serving customers

**Example:**

```csv
Time,Event,CustomerID,QueueLength,BusyServers
0.00,Arrival,0,1,0
0.00,ServiceStart,0,0,1
25.00,Arrival,1,1,1
30.00,ServiceEnd,0,1,0
```

**Streaming Benefits:**
- ✅ Events written to disk immediately as they occur
- ✅ No memory accumulation for event history
- ✅ Safe for very long simulations
- ✅ Can be analyzed/plotted while simulation is still running

## Performance Considerations

**Memory Efficiency (✨ New Streaming Architecture):**

The simulation now uses a **streaming architecture** that minimizes memory usage:

- ✅ **Events streamed to CSV** - Events written to disk as they occur, not stored in memory
- ✅ **Incremental statistics** - Running averages calculated on-the-fly during simulation
- ✅ **No event history buffer** - Only current state kept in memory, enabling very long simulations
- ⚠️ **Customer pre-generation** - All customers are still generated upfront (see warning system below)

**For Very Large Simulations:**

The system now warns you when generating >100,000 customers:
```
⚠️  WARNING: Estimated 1576800 customers will be generated!
    This may consume significant memory and time.
    Generating customers...
    Generated 100000 customers...
    ✓ Generation complete: 1576800 customers created
```

**Memory Estimate:** ~100 bytes per customer (e.g., 1 million customers ≈ 100 MB)

**Clock Precision:** Uses f64 for time representation, suitable for most simulation scenarios

## License

This project is available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Future Enhancements

Completed:
- [x] ~~Streaming CSV output~~ - ✅ Implemented with real-time event writing
- [x] ~~Incremental statistics calculation~~ - ✅ No need to store all events in memory

Potential improvements:
- [ ] Add priority queue support
- [ ] Implement customer classes with different service requirements
- [ ] Add visualization/plotting capabilities
- [ ] Support for time-varying arrival rates
- [ ] Parallel simulation runs for Monte Carlo analysis
- [ ] Web-based dashboard for real-time monitoring
- [ ] Database export in addition to CSV

## Author

Created as a discrete-event simulation framework for studying queueing systems.

## Acknowledgments

Built using Rust's powerful async/await system and demonstrates practical applications of:
- Discrete-event simulation techniques
- Custom Future implementations
- Shared state management in async contexts
- Configuration-driven application design

