#!/usr/bin/env python3
"""
Regression Detection System for Broken Divinity
Automated detection of regressions in game behavior and performance
"""

import json
import sqlite3
import time
import psutil
import numpy as np
import pandas as pd
from dataclasses import dataclass, asdict
from typing import Dict, List, Optional, Tuple, Any
from enum import Enum
import logging
from pathlib import Path
import subprocess
import re
import statistics
from datetime import datetime, timedelta
import hashlib
import os

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class RegressionType(Enum):
    """Types of regressions that can be detected"""
    PERFORMANCE_REGRESSION = "performance_regression"
    FUNCTIONAL_REGRESSION = "functional_regression"
    BALANCE_REGRESSION = "balance_regression"
    MEMORY_REGRESSION = "memory_regression"
    CRASH_REGRESSION = "crash_regression"
    UI_REGRESSION = "ui_regression"

@dataclass
class Regression:
    """Represents a regression found during analysis"""
    regression_type: RegressionType
    severity: float  # 0.0 to 1.0
    description: str
    affected_components: List[str]
    baseline_metrics: Dict[str, Any]
    current_metrics: Dict[str, Any]
    change_percentage: float
    confidence: float  # 0.0 to 1.0
    timestamp: str

@dataclass
class RegressionMetrics:
    """Comprehensive regression metrics for analysis"""
    performance_metrics: Dict[str, float]
    functional_metrics: Dict[str, float]
    memory_metrics: Dict[str, float]
    crash_metrics: Dict[str, float]
    regressions_found: List[Regression]
    baseline_hash: str
    current_hash: str
    analysis_duration: float

class RegressionDetectionSystem:
    """Main regression detection system for Broken Divinity"""
    
    def __init__(self, game_path: str, db_path: str = "testing/metrics.db"):
        self.game_path = game_path
        self.db_path = db_path
        self.regressions: List[Regression] = []
        self.metrics: Optional[RegressionMetrics] = None
        
        # Initialize database
        self._init_database()
        
        # Regression thresholds for detection
        self.thresholds = {
            'performance_degradation': 0.2,  # 20% performance degradation
            'memory_increase': 0.3,         # 30% memory increase
            'crash_rate_increase': 0.1,     # 10% crash rate increase
            'functional_failure': 0.05,     # 5% functional failure rate
            'balance_change': 0.15,         # 15% balance change
        }
        
        # Baseline data storage
        self.baseline_dir = Path("testing/baseline_data")
        self.baseline_dir.mkdir(exist_ok=True)
        
        logger.info("Regression Detection System initialized")
    
    def _init_database(self):
        """Initialize SQLite database for storing regression analysis results"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Create regression analysis table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS regression_analysis (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT,
                    baseline_hash TEXT,
                    current_hash TEXT,
                    analysis_duration REAL,
                    regressions_count INTEGER,
                    performance_score REAL,
                    functional_score REAL,
                    memory_score REAL,
                    crash_score REAL,
                    metrics_json TEXT
                )
            ''')
            
            # Create regression issues table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS regression_issues (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    analysis_id INTEGER,
                    regression_type TEXT,
                    severity REAL,
                    description TEXT,
                    affected_components TEXT,
                    baseline_metrics TEXT,
                    current_metrics TEXT,
                    change_percentage REAL,
                    confidence REAL,
                    timestamp TEXT,
                    FOREIGN KEY (analysis_id) REFERENCES regression_analysis (id)
                )
            ''')
            
            conn.commit()
            conn.close()
            logger.info("Database initialized successfully")
            
        except Exception as e:
            logger.error(f"Database initialization failed: {e}")
            raise
    
    def establish_baseline(self, scenarios: List[str] = None, runs: int = 5) -> str:
        """Establish baseline performance metrics"""
        if scenarios is None:
            scenarios = ['quick_cycle', 'new_game']
        
        logger.info(f"Establishing baseline with scenarios: {scenarios}")
        
        baseline_data = {
            'performance': [],
            'functional': [],
            'memory': [],
            'crash': [],
            'timestamp': datetime.now().isoformat()
        }
        
        try:
            # Run multiple times to establish baseline
            for run in range(runs):
                logger.info(f"Baseline run {run + 1}/{runs}")
                
                for scenario in scenarios:
                    result = subprocess.run([
                        'python3', 'testing/session_orchestrator.py', 
                        self.game_path, '--scenario', scenario, '--log-level', 'DEBUG'
                    ], capture_output=True, text=True, timeout=30)
                    
                    run_metrics = self._extract_run_metrics(result, scenario)
                    baseline_data['performance'].append(run_metrics['performance'])
                    baseline_data['functional'].append(run_metrics['functional'])
                    baseline_data['memory'].append(run_metrics['memory'])
                    baseline_data['crash'].append(run_metrics['crash'])
            
            # Calculate baseline statistics
            baseline_metrics = {
                'performance': {
                    'avg_time': statistics.mean([m['execution_time'] for m in baseline_data['performance']]),
                    'avg_memory': statistics.mean([m['memory_usage'] for m in baseline_data['memory']]),
                    'success_rate': statistics.mean([m['success'] for m in baseline_data['functional']]),
                    'crash_rate': 1 - statistics.mean([m['success'] for m in baseline_data['crash']])
                },
                'timestamp': baseline_data['timestamp']
            }
            
            # Generate baseline hash
            baseline_hash = self._generate_hash(baseline_metrics)
            
            # Store baseline data
            baseline_file = self.baseline_dir / f"baseline_{baseline_hash}.json"
            with open(baseline_file, 'w') as f:
                json.dump(baseline_metrics, f, indent=2)
            
            logger.info(f"Baseline established with hash: {baseline_hash}")
            return baseline_hash
            
        except Exception as e:
            logger.error(f"Baseline establishment failed: {e}")
            raise
    
    def detect_regressions(self, baseline_hash: str = None, scenarios: List[str] = None, 
                          runs: int = 3) -> RegressionMetrics:
        """Detect regressions against baseline"""
        if scenarios is None:
            scenarios = ['quick_cycle', 'new_game']
        
        logger.info(f"Detecting regressions with scenarios: {scenarios}")
        start_time = time.time()
        
        try:
            # Load baseline data
            if baseline_hash:
                baseline_file = self.baseline_dir / f"baseline_{baseline_hash}.json"
                if baseline_file.exists():
                    with open(baseline_file, 'r') as f:
                        baseline_metrics = json.load(f)
                else:
                    logger.warning(f"Baseline file not found: {baseline_file}")
                    baseline_metrics = None
            else:
                baseline_metrics = None
            
            # Collect current performance data
            current_data = {
                'performance': [],
                'functional': [],
                'memory': [],
                'crash': [],
                'timestamp': datetime.now().isoformat()
            }
            
            for run in range(runs):
                logger.info(f"Current run {run + 1}/{runs}")
                
                for scenario in scenarios:
                    result = subprocess.run([
                        'python3', 'testing/session_orchestrator.py', 
                        self.game_path, '--scenario', scenario, '--log-level', 'DEBUG'
                    ], capture_output=True, text=True, timeout=30)
                    
                    run_metrics = self._extract_run_metrics(result, scenario)
                    current_data['performance'].append(run_metrics['performance'])
                    current_data['functional'].append(run_metrics['functional'])
                    current_data['memory'].append(run_metrics['memory'])
                    current_data['crash'].append(run_metrics['crash'])
            
            # Calculate current metrics
            current_metrics = {
                'performance': {
                    'avg_time': statistics.mean([m['execution_time'] for m in current_data['performance']]),
                    'avg_memory': statistics.mean([m['memory_usage'] for m in current_data['memory']]),
                    'success_rate': statistics.mean([m['success'] for m in current_data['functional']]),
                    'crash_rate': 1 - statistics.mean([m['success'] for m in current_data['crash']])
                },
                'timestamp': current_data['timestamp']
            }
            
            # Generate current hash
            current_hash = self._generate_hash(current_metrics)
            
            # Detect regressions
            self.regressions = self._analyze_regressions(baseline_metrics, current_metrics)
            
            # Calculate component scores
            performance_score = self._calculate_performance_score(baseline_metrics, current_metrics)
            functional_score = self._calculate_functional_score(baseline_metrics, current_metrics)
            memory_score = self._calculate_memory_score(baseline_metrics, current_metrics)
            crash_score = self._calculate_crash_score(baseline_metrics, current_metrics)
            
            # Create regression metrics
            self.metrics = RegressionMetrics(
                performance_metrics=current_metrics['performance'],
                functional_metrics=current_metrics['functional'],
                memory_metrics=current_metrics['memory'],
                crash_metrics=current_metrics['crash'],
                regressions_found=self.regressions,
                baseline_hash=baseline_hash or "none",
                current_hash=current_hash,
                analysis_duration=time.time() - start_time
            )
            
            # Store results in database
            self._store_regression_results(self.metrics)
            
            logger.info(f"Regression detection completed in {self.metrics.analysis_duration:.2f} seconds")
            logger.info(f"Regressions found: {len(self.regressions)}")
            
            return self.metrics
            
        except Exception as e:
            logger.error(f"Regression detection failed: {e}")
            raise
    
    def _extract_run_metrics(self, result: subprocess.CompletedProcess, scenario: str) -> Dict[str, Any]:
        """Extract metrics from a single run"""
        metrics = {
            'performance': {},
            'functional': {},
            'memory': {},
            'crash': {}
        }
        
        # Extract execution time
        time_match = re.search(r'Duration: (\d+\.\d+) seconds', result.stdout)
        if time_match:
            metrics['performance']['execution_time'] = float(time_match.group(1))
        else:
            metrics['performance']['execution_time'] = 0
        
        # Extract memory usage
        memory_match = re.search(r'Memory: (\d+) KB', result.stdout)
        if memory_match:
            metrics['memory']['memory_usage'] = float(memory_match.group(1)) / 1024  # Convert to MB
        else:
            metrics['memory']['memory_usage'] = 0
        
        # Extract success/failure
        if result.returncode == 0:
            metrics['functional']['success'] = 1
            metrics['crash']['success'] = 1
        else:
            metrics['functional']['success'] = 0
            metrics['crash']['success'] = 0
        
        # Extract scenario-specific metrics
        if scenario == 'quick_cycle':
            metrics['performance']['scenario'] = 'quick_cycle'
        elif scenario == 'new_game':
            metrics['performance']['scenario'] = 'new_game'
        else:
            metrics['performance']['scenario'] = 'unknown'
        
        return metrics
    
    def _analyze_regressions(self, baseline_metrics: Dict, current_metrics: Dict) -> List[Regression]:
        """Analyze regressions between baseline and current metrics"""
        regressions = []
        
        if not baseline_metrics:
            logger.warning("No baseline metrics available for comparison")
            return regressions
        
        # Performance regression detection
        if current_metrics['performance']['avg_time'] > baseline_metrics['performance']['avg_time']:
            time_change = (current_metrics['performance']['avg_time'] - baseline_metrics['performance']['avg_time']) / baseline_metrics['performance']['avg_time']
            
            if time_change > self.thresholds['performance_degradation']:
                regressions.append(Regression(
                    regression_type=RegressionType.PERFORMANCE_REGRESSION,
                    severity=min(1.0, time_change),
                    description=f"Performance degraded by {time_change:.1%}",
                    affected_components=['execution_time', 'game_loop'],
                    baseline_metrics={'avg_time': baseline_metrics['performance']['avg_time']},
                    current_metrics={'avg_time': current_metrics['performance']['avg_time']},
                    change_percentage=time_change * 100,
                    confidence=0.9,
                    timestamp=datetime.now().isoformat()
                ))
        
        # Memory regression detection
        if current_metrics['performance']['avg_memory'] > baseline_metrics['performance']['avg_memory']:
            memory_change = (current_metrics['performance']['avg_memory'] - baseline_metrics['performance']['avg_memory']) / baseline_metrics['performance']['avg_memory']
            
            if memory_change > self.thresholds['memory_increase']:
                regressions.append(Regression(
                    regression_type=RegressionType.MEMORY_REGRESSION,
                    severity=min(1.0, memory_change),
                    description=f"Memory usage increased by {memory_change:.1%}",
                    affected_components=['memory_management', 'resource_loading'],
                    baseline_metrics={'avg_memory': baseline_metrics['performance']['avg_memory']},
                    current_metrics={'avg_memory': current_metrics['performance']['avg_memory']},
                    change_percentage=memory_change * 100,
                    confidence=0.8,
                    timestamp=datetime.now().isoformat()
                ))
        
        # Functional regression detection
        if current_metrics['performance']['success_rate'] < baseline_metrics['performance']['success_rate']:
            success_change = (baseline_metrics['performance']['success_rate'] - current_metrics['performance']['success_rate']) / baseline_metrics['performance']['success_rate']
            
            if success_change > self.thresholds['functional_failure']:
                regressions.append(Regression(
                    regression_type=RegressionType.FUNCTIONAL_REGRESSION,
                    severity=min(1.0, success_change),
                    description=f"Success rate decreased by {success_change:.1%}",
                    affected_components=['game_logic', 'state_management'],
                    baseline_metrics={'success_rate': baseline_metrics['performance']['success_rate']},
                    current_metrics={'success_rate': current_metrics['performance']['success_rate']},
                    change_percentage=success_change * 100,
                    confidence=0.9,
                    timestamp=datetime.now().isoformat()
                ))
        
        # Crash regression detection
        if current_metrics['performance']['crash_rate'] > baseline_metrics['performance']['crash_rate']:
            crash_change = (current_metrics['performance']['crash_rate'] - baseline_metrics['performance']['crash_rate']) / baseline_metrics['performance']['crash_rate']
            
            if crash_change > self.thresholds['crash_rate_increase']:
                regressions.append(Regression(
                    regression_type=RegressionType.CRASH_REGRESSION,
                    severity=min(1.0, crash_change),
                    description=f"Crash rate increased by {crash_change:.1%}",
                    affected_components=['error_handling', 'stability'],
                    baseline_metrics={'crash_rate': baseline_metrics['performance']['crash_rate']},
                    current_metrics={'crash_rate': current_metrics['performance']['crash_rate']},
                    change_percentage=crash_change * 100,
                    confidence=0.8,
                    timestamp=datetime.now().isoformat()
                ))
        
        return regressions
    
    def _calculate_performance_score(self, baseline_metrics: Dict, current_metrics: Dict) -> float:
        """Calculate performance score"""
        if not baseline_metrics:
            return 0.5
        
        time_ratio = current_metrics['performance']['avg_time'] / baseline_metrics['performance']['avg_time']
        memory_ratio = current_metrics['performance']['avg_memory'] / baseline_metrics['performance']['avg_memory']
        
        # Performance score inversely related to degradation
        time_score = max(0, 1 - (time_ratio - 1))
        memory_score = max(0, 1 - (memory_ratio - 1))
        
        return (time_score + memory_score) / 2
    
    def _calculate_functional_score(self, baseline_metrics: Dict, current_metrics: Dict) -> float:
        """Calculate functional score"""
        if not baseline_metrics:
            return current_metrics['performance']['success_rate']
        
        return current_metrics['performance']['success_rate']
    
    def _calculate_memory_score(self, baseline_metrics: Dict, current_metrics: Dict) -> float:
        """Calculate memory score"""
        if not baseline_metrics:
            return 0.5
        
        memory_ratio = current_metrics['performance']['avg_memory'] / baseline_metrics['performance']['avg_memory']
        return max(0, 1 - (memory_ratio - 1))
    
    def _calculate_crash_score(self, baseline_metrics: Dict, current_metrics: Dict) -> float:
        """Calculate crash score"""
        if not baseline_metrics:
            return 1 - current_metrics['performance']['crash_rate']
        
        return 1 - current_metrics['performance']['crash_rate']
    
    def _generate_hash(self, metrics: Dict) -> str:
        """Generate hash for metrics"""
        metrics_str = json.dumps(metrics, sort_keys=True)
        return hashlib.md5(metrics_str.encode()).hexdigest()
    
    def _store_regression_results(self, metrics: RegressionMetrics):
        """Store regression results in database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Store main regression results
            cursor.execute('''
                INSERT INTO regression_analysis 
                (timestamp, baseline_hash, current_hash, analysis_duration, regressions_count,
                 performance_score, functional_score, memory_score, crash_score, metrics_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                metrics.timestamp,
                metrics.baseline_hash,
                metrics.current_hash,
                metrics.analysis_duration,
                len(metrics.regressions_found),
                self._calculate_performance_score(None, metrics.performance_metrics),
                self._calculate_functional_score(None, metrics.functional_metrics),
                self._calculate_memory_score(None, metrics.memory_metrics),
                self._calculate_crash_score(None, metrics.crash_metrics),
                json.dumps(asdict(metrics))
            ))
            
            analysis_id = cursor.lastrowid
            
            # Store individual regressions
            for regression in metrics.regressions_found:
                cursor.execute('''
                    INSERT INTO regression_issues 
                    (analysis_id, regression_type, severity, description, affected_components,
                     baseline_metrics, current_metrics, change_percentage, confidence, timestamp)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    analysis_id,
                    regression.regression_type.value,
                    regression.severity,
                    regression.description,
                    json.dumps(regression.affected_components),
                    json.dumps(regression.baseline_metrics),
                    json.dumps(regression.current_metrics),
                    regression.change_percentage,
                    regression.confidence,
                    regression.timestamp
                ))
            
            conn.commit()
            conn.close()
            
        except Exception as e:
            logger.error(f"Failed to store regression results: {e}")
    
    def generate_report(self, output_format: str = 'json') -> str:
        """Generate regression detection report"""
        if not self.metrics:
            return "No regression data available"
        
        if output_format == 'json':
            return json.dumps(asdict(self.metrics), indent=2)
        elif output_format == 'html':
            return self._generate_html_report()
        else:
            return self._generate_text_report()
    
    def _generate_text_report(self) -> str:
        """Generate text-based regression report"""
        if not self.metrics:
            return "No regression data available"
        
        report = []
        report.append("=" * 60)
        report.append("BROKEN DIVINITY REGRESSION DETECTION REPORT")
        report.append("=" * 60)
        report.append(f"Analysis Time: {self.metrics.timestamp}")
        report.append(f"Baseline Hash: {self.metrics.baseline_hash}")
        report.append(f"Current Hash: {self.metrics.current_hash}")
        report.append(f"Analysis Duration: {self.metrics.analysis_duration:.2f} seconds")
        report.append("")
        
        # Performance Metrics
        report.append("PERFORMANCE METRICS")
        report.append("-" * 20)
        perf = self.metrics.performance_metrics
        report.append(f"Average Execution Time: {perf['avg_time']:.2f} seconds")
        report.append(f"Average Memory Usage: {perf['avg_memory']:.2f} MB")
        report.append(f"Success Rate: {perf['success_rate']:.2%}")
        report.append(f"Crash Rate: {perf['crash_rate']:.2%}")
        report.append("")
        
        # Component Scores
        report.append("COMPONENT SCORES")
        report.append("-" * 15)
        report.append(f"Performance Score: {self._calculate_performance_score(None, self.metrics.performance_metrics):.2f}")
        report.append(f"Functional Score: {self._calculate_functional_score(None, self.metrics.functional_metrics):.2f}")
        report.append(f"Memory Score: {self._calculate_memory_score(None, self.metrics.memory_metrics):.2f}")
        report.append(f"Crash Score: {self._calculate_crash_score(None, self.metrics.crash_metrics):.2f}")
        report.append("")
        
        # Regressions
        report.append("REGRESSIONS DETECTED")
        report.append("-" * 18)
        report.append(f"Total Regressions: {len(self.metrics.regressions_found)}")
        report.append("")
        
        for i, regression in enumerate(self.metrics.regressions_found, 1):
            report.append(f"{i}. {regression.regression_type.value.upper()}")
            report.append(f"   Severity: {regression.severity:.2f}")
            report.append(f"   Description: {regression.description}")
            report.append(f"   Affected Components: {', '.join(regression.affected_components)}")
            report.append(f"   Change Percentage: {regression.change_percentage:.2f}%")
            report.append(f"   Confidence: {regression.confidence:.2f}")
            report.append("")
        
        return "\n".join(report)
    
    def _generate_html_report(self) -> str:
        """Generate HTML-based regression report"""
        if not self.metrics:
            return "<html><body><h1>No regression data available</h1></body></html>"
        
        html = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Broken Divinity Regression Detection Report</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
                .section {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
                .metric {{ margin: 10px 0; }}
                .regression {{ background-color: #f8d7da; padding: 10px; margin: 10px 0; border-radius: 3px; }}
                .severity-high {{ color: #dc3545; }}
                .severity-medium {{ color: #fd7e14; }}
                .severity-low {{ color: #28a745; }}
            </style>
        </head>
        <body>
            <div class="header">
                <h1>Broken Divinity Regression Detection Report</h1>
                <p>Analysis Time: {self.metrics.timestamp}</p>
                <p>Baseline Hash: {self.metrics.baseline_hash}</p>
                <p>Current Hash: {self.metrics.current_hash}</p>
                <p>Analysis Duration: {self.metrics.analysis_duration:.2f} seconds</p>
            </div>
            
            <div class="section">
                <h2>Performance Metrics</h2>
                <div class="metric">Average Execution Time: {self.metrics.performance_metrics['avg_time']:.2f} seconds</div>
                <div class="metric">Average Memory Usage: {self.metrics.performance_metrics['avg_memory']:.2f} MB</div>
                <div class="metric">Success Rate: {self.metrics.performance_metrics['success_rate']:.2%}</div>
                <div class="metric">Crash Rate: {self.metrics.performance_metrics['crash_rate']:.2%}</div>
            </div>
            
            <div class="section">
                <h2>Component Scores</h2>
                <div class="metric">Performance Score: {self._calculate_performance_score(None, self.metrics.performance_metrics):.2f}</div>
                <div class="metric">Functional Score: {self._calculate_functional_score(None, self.metrics.functional_metrics):.2f}</div>
                <div class="metric">Memory Score: {self._calculate_memory_score(None, self.metrics.memory_metrics):.2f}</div>
                <div class="metric">Crash Score: {self._calculate_crash_score(None, self.metrics.crash_metrics):.2f}</div>
            </div>
            
            <div class="section">
                <h2>Regressions Detected</h2>
                <p>Total Regressions: {len(self.metrics.regressions_found)}</p>
        """
        
        for regression in self.metrics.regressions_found:
            severity_class = f"severity-{regression.severity:.2f}"
            html += f"""
                <div class="regression">
                    <h3>{regression.regression_type.value.upper()}</h3>
                    <p><strong>Severity:</strong> <span class="{severity_class}">{regression.severity:.2f}</span></p>
                    <p><strong>Description:</strong> {regression.description}</p>
                    <p><strong>Affected Components:</strong> {', '.join(regression.affected_components)}</p>
                    <p><strong>Change Percentage:</strong> {regression.change_percentage:.2f}%</p>
                    <p><strong>Confidence:</strong> {regression.confidence:.2f}</p>
                </div>
            """
        
        html += """
            </div>
        </body>
        </html>
        """
        
        return html

def main():
    """Main function for running regression detection"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Regression Detection System for Broken Divinity')
    parser.add_argument('game_path', help='Path to the game binary')
    parser.add_argument('--baseline', help='Baseline hash to compare against')
    parser.add_argument('--establish-baseline', action='store_true', help='Establish new baseline')
    parser.add_argument('--scenarios', nargs='+', default=['quick_cycle'], 
                       help='Scenarios to run for analysis')
    parser.add_argument('--runs', type=int, default=3, help='Number of runs per scenario')
    parser.add_argument('--output', choices=['json', 'text', 'html'], default='text',
                       help='Output format')
    parser.add_argument('--output-file', help='Output file path')
    parser.add_argument('--verbose', action='store_true', help='Verbose logging')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        # Initialize regression detection system
        system = RegressionDetectionSystem(args.game_path)
        
        # Establish baseline if requested
        if args.establish_baseline:
            baseline_hash = system.establish_baseline(args.scenarios, args.runs)
            print(f"Baseline established with hash: {baseline_hash}")
            exit(0)
        
        # Detect regressions
        metrics = system.detect_regressions(args.baseline, args.scenarios, args.runs)
        
        # Generate report
        if args.output == 'json':
            report = system.generate_report('json')
        elif args.output == 'html':
            report = system.generate_report('html')
        else:
            report = system.generate_report('text')
        
        # Output report
        if args.output_file:
            with open(args.output_file, 'w') as f:
                f.write(report)
            print(f"Report saved to {args.output_file}")
        else:
            print(report)
        
        # Exit with appropriate code
        if len(metrics.regressions_found) > 0:
            exit(1)  # Regressions found
        else:
            exit(0)  # No regressions found
            
    except Exception as e:
        logger.error(f"Regression detection failed: {e}")
        exit(2)

if __name__ == "__main__":
    main()