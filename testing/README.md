# Broken Divinity Testing Framework

A comprehensive testing framework for Broken Divinity that provides automated testing, analytics, and quality assurance capabilities.

## Overview

This testing framework is designed to validate the game's functionality, performance, and balance through automated testing tools. It includes:

- **Week 1 Foundation**: Core testing infrastructure
- **Week 2 Core Tools**: Advanced analytics and validation tools

## Quick Start

### Prerequisites

- Python 3.8+
- Required packages: `pandas`, `numpy`, `psutil`

Install dependencies:
```bash
pip install pandas numpy psutil
```

## Structure

```
testing/
├── README.md                    # This file
├── run_all_tests.py             # Comprehensive test runner
├── cli_wrapper.py              # CLI interface wrapper
├── session_orchestrator.py     # Test scenario orchestration
├── test_framework.py           # Core testing framework
├── metrics_collector.py       # Performance metrics collection
├── balance_analytics.py        # Game balance analysis
├── regression_detection.py     # Regression detection system
├── test_data_generator.py      # Test data generation
├── integration_testing.py     # Integration validation
├── metrics.db                  # SQLite database for metrics
└── requirements.txt           # Python dependencies
```

## Installation

1. Install Python 3.8 or higher
2. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

## Usage

### Running All Tests

```bash
python3 testing/run_all_tests.py /path/to/broken_divinity --duration 5 --output results.json
```

### Individual Tools

#### CLI Wrapper
```bash
python3 testing/cli_wrapper.py /path/to/broken_divinity --duration 10 --verbose
```

#### Session Orchestrator
```bash
python3 testing/session_orchestrator.py /path/to/broken_divinity --scenario quick_cycle --verbose
```

#### Test Framework
```bash
python3 testing/test_framework.py --verbose
```

#### Metrics Collector
```bash
python3 testing/metrics_collector.py --verbose
```

#### Balance Analytics
```bash
python3 testing/balance_analytics.py /path/to/broken_divinity --duration 10 --verbose
```

#### Regression Detection
```bash
python3 testing/regression_detection.py /path/to/broken_divinity --runs 3 --verbose
```

#### Test Data Generator
```bash
python3 testing/test_data_generator.py /path/to/broken_divinity --profile complex --count 50 --verbose
```

#### Integration Testing
```bash
python3 testing/integration_testing.py /path/to/broken_divinity --test-types tool_chain data_flow performance --parallel --verbose
```

## Tool Descriptions

### Week 1 Foundation

#### CLI Wrapper (`cli_wrapper.py`)
- **Purpose**: Provides a command-line interface to the game
- **Features**:
  - Headless mode execution
  - Game status monitoring
  - Exit code handling
  - Comprehensive logging
- **Usage**: Primary interface for automated game execution

#### Session Orchestrator (`session_orchestrator.py`)
- **Purpose**: Orchestrates test scenarios and game sessions
- **Features**:
  - Predefined scenarios (quick_cycle, full_session, stress_test)
  - Step-by-step execution
  - Session state management
  - Comprehensive logging
- **Usage**: Manages game sessions with specific test scenarios

#### Test Framework (`test_framework.py`)
- **Purpose**: Core testing framework with Bevy integration
- **Features**:
  - Bevy 0.18 integration
  - Test discovery and execution
  - Mock and real game testing
  - Comprehensive reporting
- **Usage**: Foundation for all automated testing

#### Metrics Collector (`metrics_collector.py`)
- **Purpose**: Collects and analyzes performance metrics
- **Features**:
  - System metrics collection
  - Test metrics recording
  - Performance analysis
  - Database storage
- **Usage**: Monitors game performance during testing

### Week 2 Core Tools

#### Balance Analytics (`balance_analytics.py`)
- **Purpose**: Analyzes game balance and mechanics
- **Features**:
  - Statistical analysis of combat mechanics
  - Progression system evaluation
  - Economic balance assessment
  - Issue detection and reporting
- **Usage**: Validates game balance and identifies balance issues

#### Regression Detection (`regression_detection.py`)
- **Purpose**: Detects performance and functionality regressions
- **Features**:
  - Baseline establishment
  - Automated regression detection
  - Performance comparison
  - Alert generation
- **Usage**: Ensures game performance doesn't degrade over time

#### Test Data Generator (`test_data_generator.py`)
- **Purpose**: Generates realistic test data for all game systems
- **Features**:
  - Multiple generation profiles
  - Realistic data creation
  - Export to multiple formats
  - Comprehensive data types
- **Usage**: Creates test data for comprehensive testing

#### Integration Testing (`integration_testing.py`)
- **Purpose**: Validates that all tools work together consistently
- **Features**:
  - Tool chain validation
  - Data flow verification
  - Performance testing
  - Consistency checking
  - Compatibility validation
- **Usage**: Ensures all testing tools work together properly

### 1. CLI Wrapper (`cli_wrapper.py`)

Run the game in headless mode for automated testing.

**Features:**
- Execute game with `--headless` flag
- Timeout handling and process management
- Result capture and analysis
- Predefined scenarios

**Usage:**
```bash
python3 cli_wrapper.py <game_path> [options]
```

**Options:**
- `--duration`: Maximum runtime in seconds (default: 30)
- `--scenario`: Run predefined scenario (new_game, quick_test, save_load_test)
- `--output`: Output file for results
- `--verbose`: Verbose output

### 2. Session Orchestrator (`session_orchestrator.py`)

Execute predefined gameplay scenarios with multiple steps.

**Features:**
- Multi-step scenario execution
- Automated game state transitions
- Comprehensive logging
- Result aggregation

**Scenarios:**
- `new_game`: Basic game startup and initialization
- `quick_cycle`: Quick game cycle to test basic functionality
- `save_load_test`: Test save and load functionality
- `stress_test`: Stress test with multiple rapid cycles

**Usage:**
```bash
python3 session_orchestrator.py <game_path> [options]
```

**Options:**
- `--scenario`: Run specific scenario
- `--all-scenarios`: Run all scenarios
- `--output`: Output file for results
- `--log-level`: Logging level (DEBUG, INFO, WARNING, ERROR)

### 3. Test Framework (`test_framework.py`)

Base testing infrastructure for automated game testing.

**Features:**
- Base test classes and inheritance
- Sequential and parallel test execution
- Comprehensive test result tracking
- Export capabilities (JSON, CSV)

**Usage:**
```bash
python3 test_framework.py [options]
```

**Options:**
- `--game-path`: Path to the game binary
- `--duration`: Test duration in seconds
- `--parallel`: Run tests in parallel
- `--output`: Output file for results
- `--verbose`: Verbose output

### 4. Metrics Collector (`metrics_collector.py`)

Collect and analyze test execution metrics.

**Features:**
- System metrics collection (CPU, memory, etc.)
- Test-specific metrics tracking
- Trend analysis
- Export capabilities (JSON, CSV)

**Usage:**
```bash
python3 metrics_collector.py [options]
```

**Options:**
- `--game-path`: Path to the game binary
- `--duration`: Test duration in seconds
- `--output`: Output file for results
- `--format`: Output format (json, csv)
- `--test-name`: Specific test name to analyze
- `--cleanup`: Clean up metrics older than N days

### 5. Integration Script (`run_all_tests.py`)

Run all testing tools together for comprehensive testing.

**Features:**
- Execute all tools in sequence
- Generate comprehensive reports
- Success rate calculation
- Export aggregated results

**Usage:**
```bash
python3 run_all_tests.py <game_path> [options]
```

**Options:**
- `--duration`: Test duration in seconds
- `--output`: Output file for results
- `--verbose`: Verbose output

## Architecture

### Core Components

1. **CLI Wrapper**
   - `CLIWrapper` class for game execution
   - `GameStatus` enum for execution states
   - `GameResult` dataclass for results

2. **Session Orchestrator**
   - `SessionOrchestrator` class for scenario management
   - `ScenarioStatus` enum for scenario states
   - `ScenarioResult` dataclass for results

3. **Test Framework**
   - `BaseTest` abstract base class
   - `TestOrchestrator` for test execution
   - `TestResult` dataclass for results
   - `SimpleTest` for function-based tests

4. **Metrics Collector**
   - `MetricsCollector` class for data collection
   - `MetricData` dataclass for individual metrics
   - SQLite database for persistent storage

### Data Flow

```
Game Binary → CLI Wrapper → Session Orchestrator → Test Framework → Metrics Collector
     ↓              ↓              ↓              ↓              ↓
 Headless Mode → Scenarios → Tests → Metrics → Reports
```

## Configuration

### Environment Variables

- `BROKEN_DIVINITY_PATH`: Default path to game binary
- `TEST_DURATION`: Default test duration in seconds
- `LOG_LEVEL`: Default logging level

### Configuration Files

Create `testing_config.json` for custom configuration:
```json
{
  "game_path": "./target/debug/broken_divinity",
  "default_duration": 10,
  "max_workers": 4,
  "log_level": "INFO",
  "storage_path": "./metrics"
}
```

## Output Formats

### JSON Output

```json
{
  "timestamp": 1623840000.0,
  "status": "completed",
  "duration": 5.23,
  "metrics": {
    "cpu_usage": 25.5,
    "memory_usage": 1024.0,
    "success_rate": 0.8
  }
}
```

### CSV Output

```csv
test_name,status,duration,timestamp,cpu_usage,memory_usage
game_test,passed,5.23,1623840000.0,25.5,1024.0
quick_test,failed,2.15,1623840005.23,30.1,2048.0
```

## Development

### Adding New Tests

1. Create a new test class inheriting from `BaseTest`
2. Implement `setup()`, `execute()`, and `teardown()` methods
3. Add test to orchestrator using `add_test()`

```python
class MyGameTest(BaseTest):
    def setup(self):
        # Setup test environment
        pass
    
    def execute(self) -> TestResult:
        # Execute test logic
        return TestResult(self.name, TestStatus.PASSED, 0, time.time(), time.time())
    
    def teardown(self):
        # Cleanup test environment
        pass
```

### Adding New Scenarios

1. Define scenario in `SessionOrchestrator._define_scenarios()`
2. Add steps with appropriate actions
3. Test scenario using `run_scenario()`

```python
"my_scenario": {
    "description": "My custom scenario",
    "duration": 15,
    "steps": [
        {"name": "step1", "action": "start_game", "expected": "game_starts"},
        {"name": "step2", "action": "run_seconds", "duration": 5},
        {"name": "step3", "action": "stop_game", "expected": "game_exits"}
    ]
}
```

### Adding New Metrics

1. Use `record_metric()` to collect data
2. Define metric type and tags
3. Analyze using `generate_report()` or `get_trends()`

```python
collector.record_metric(
    name="my_custom_metric",
    value=42,
    metric_type=MetricType.GAUGE,
    tags={"test_name": "my_test", "category": "performance"}
)
```

## Troubleshooting

### Common Issues

1. **Game crashes in headless mode**
   - Expected behavior - game is not fully configured for headless testing
   - Framework handles crashes gracefully and reports them

2. **Import errors**
   - Ensure all dependencies are installed: `pip install pandas numpy psutil`
   - Check Python path and working directory

3. **Permission errors**
   - Ensure game binary is executable: `chmod +x ./target/debug/broken_divinity`
   - Check file permissions for output files

### Debug Mode

Enable verbose logging for detailed debugging:
```bash
python3 testing/run_all_tests.py ./target/debug/broken_divinity --verbose
```

### Log Files

Check `testing.log` for detailed execution logs.

## Future Enhancements

1. **Enhanced Game Integration**
   - Direct game state inspection
   - Input injection for automated gameplay
   - Save/load state manipulation

2. **Advanced Analytics**
   - Statistical analysis and significance testing
   - Performance regression detection
   - Balance validation

3. **CI/CD Integration**
   - GitHub Actions workflows
   - Automated testing on commits
   - Performance monitoring

4. **Alert System**
   - Email/Slack notifications
   - Custom alert rules
   - Escalation procedures

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This testing framework is part of the Broken Divinity project and follows the same license terms.