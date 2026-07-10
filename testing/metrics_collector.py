#!/usr/bin/env python3
"""
Broken Divinity Metrics Collector
Basic data logging and analysis for testing framework
"""

import time
import json
import psutil
import os
import logging
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
import pandas as pd
import numpy as np
from datetime import datetime
import threading
import subprocess
from concurrent.futures import ThreadPoolExecutor
import sqlite3
import csv


class MetricType(Enum):
    """Types of metrics that can be collected"""
    COUNTER = "counter"
    GAUGE = "gauge"
    HISTOGRAM = "histogram"
    TIMING = "timing"


@dataclass
class MetricData:
    """Individual metric data point"""
    name: str
    value: Union[int, float]
    metric_type: MetricType
    timestamp: float
    tags: Dict[str, str] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            "name": self.name,
            "value": self.value,
            "type": self.metric_type.value,
            "timestamp": self.timestamp,
            "tags": self.tags
        }


class MetricsCollector:
    """Collect and analyze test execution metrics"""
    
    def __init__(self, storage_path: Optional[str] = None):
        self.metrics: List[MetricData] = []
        self.storage_path = Path(storage_path) if storage_path else Path("metrics")
        self.storage_path.mkdir(exist_ok=True)
        
        # Setup logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        self.logger = logging.getLogger(__name__)
        
        # Process monitoring
        self.process = psutil.Process()
        self.start_time = time.time()
        
        # Database setup
        self.db_path = self.storage_path / "metrics.db"
        self._setup_database()
    
    def _setup_database(self):
        """Setup SQLite database for metrics storage"""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute('''
                    CREATE TABLE IF NOT EXISTS metrics (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        value REAL NOT NULL,
                        type TEXT NOT NULL,
                        timestamp REAL NOT NULL,
                        tags TEXT
                    )
                ''')
                
                conn.execute('''
                    CREATE INDEX IF NOT EXISTS idx_metrics_name 
                    ON metrics(name)
                ''')
                
                conn.execute('''
                    CREATE INDEX IF NOT EXISTS idx_metrics_timestamp 
                    ON metrics(timestamp)
                ''')
                
                conn.commit()
                
        except Exception as e:
            self.logger.error(f"Database setup failed: {e}")
    
    def record_metric(self, name: str, value: Union[int, float], 
                     metric_type: MetricType = MetricType.GAUGE,
                     tags: Optional[Dict[str, str]] = None):
        """Record a metric data point"""
        timestamp = time.time()
        
        metric = MetricData(
            name=name,
            value=value,
            metric_type=metric_type,
            timestamp=timestamp,
            tags=tags or {}
        )
        
        self.metrics.append(metric)
        
        # Store in database
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute('''
                    INSERT INTO metrics (name, value, type, timestamp, tags)
                    VALUES (?, ?, ?, ?, ?)
                ''', (name, value, metric_type.value, timestamp, json.dumps(tags or {})))
                conn.commit()
                
        except Exception as e:
            self.logger.error(f"Failed to store metric: {e}")
    
    def record_test_run(self, test_name: str, duration: float, 
                       memory_usage: float, success: bool,
                       cpu_usage: Optional[float] = None,
                       additional_metrics: Optional[Dict[str, Any]] = None):
        """Record test execution metrics"""
        tags = {
            "test_name": test_name,
            "success": str(success)
        }
        
        # Record basic metrics
        self.record_metric(f"test_duration.{test_name}", duration, MetricType.TIMING, tags)
        self.record_metric(f"test_memory.{test_name}", memory_usage, MetricType.GAUGE, tags)
        self.record_metric(f"test_success.{test_name}", 1 if success else 0, MetricType.COUNTER, tags)
        
        if cpu_usage is not None:
            self.record_metric(f"test_cpu.{test_name}", cpu_usage, MetricType.GAUGE, tags)
        
        # Record additional metrics
        if additional_metrics:
            for key, value in additional_metrics.items():
                self.record_metric(f"test_additional.{test_name}.{key}", value, MetricType.GAUGE, tags)
    
    def get_system_metrics(self) -> Dict[str, float]:
        """Get current system metrics"""
        try:
            memory_info = self.process.memory_info()
            cpu_percent = self.process.cpu_percent()
            
            return {
                "memory_rss": memory_info.rss / 1024 / 1024,  # MB
                "memory_vms": memory_info.vms / 1024 / 1024,  # MB
                "cpu_percent": cpu_percent,
                "file_descriptors": len(self.process.open_files()),
                "threads": self.process.num_threads(),
                "uptime": time.time() - self.start_time
            }
        except Exception as e:
            self.logger.error(f"Failed to get system metrics: {e}")
            return {}
    
    def record_system_metrics(self, interval: float = 1.0):
        """Record system metrics at regular intervals"""
        def record_metrics():
            while True:
                metrics = self.get_system_metrics()
                for name, value in metrics.items():
                    self.record_metric(f"system_{name}", value, MetricType.GAUGE)
                time.sleep(interval)
        
        # Start background thread
        thread = threading.Thread(target=record_metrics, daemon=True)
        thread.start()
    
    def generate_report(self, test_name: Optional[str] = None) -> Dict[str, Any]:
        """Generate basic metrics report"""
        # Filter metrics by test name if specified
        if test_name:
            filtered_metrics = [m for m in self.metrics if test_name in m.name]
        else:
            filtered_metrics = self.metrics
        
        if not filtered_metrics:
            return {"message": "No metrics found"}
        
        # Convert to DataFrame for analysis
        df = pd.DataFrame([m.to_dict() for m in filtered_metrics])
        df['timestamp'] = pd.to_datetime(df['timestamp'], unit='s')
        
        # Basic statistics
        report = {
            "total_metrics": len(filtered_metrics),
            "time_range": {
                "start": df['timestamp'].min().isoformat(),
                "end": df['timestamp'].max().isoformat(),
                "duration": (df['timestamp'].max() - df['timestamp'].min()).total_seconds()
            },
            "metrics_by_type": df['type'].value_counts().to_dict(),
            "metrics_by_name": df['name'].value_counts().to_dict()
        }
        
        # Test-specific metrics
        test_metrics = df[df['name'].str.startswith('test_')]
        if not test_metrics.empty:
            report["test_metrics"] = {
                "total_tests": len(test_metrics['name'].str.extract(r'test_(.+?)\.')[0].unique()),
                "success_rate": test_metrics[test_metrics['name'].str.contains('success')]['value'].mean(),
                "avg_duration": test_metrics[test_metrics['name'].str.contains('duration')]['value'].mean(),
                "avg_memory": test_metrics[test_metrics['name'].str.contains('memory')]['value'].mean()
            }
        
        # System metrics
        system_metrics = df[df['name'].str.startswith('system_')]
        if not system_metrics.empty:
            report["system_metrics"] = {
                "avg_cpu": system_metrics[system_metrics['name'] == 'system_cpu_percent']['value'].mean(),
                "avg_memory_rss": system_metrics[system_metrics['name'] == 'system_memory_rss']['value'].mean(),
                "avg_memory_vms": system_metrics[system_metrics['name'] == 'system_memory_vms']['value'].mean()
            }
        
        return report
    
    def export_to_json(self, output_file: str, test_name: Optional[str] = None):
        """Export metrics to JSON file"""
        report = self.generate_report(test_name)
        
        with open(output_file, 'w') as f:
            json.dump(report, f, indent=2, default=str)
        
        self.logger.info(f"Metrics exported to: {output_file}")
    
    def export_to_csv(self, output_file: str, test_name: Optional[str] = None):
        """Export metrics to CSV file"""
        # Filter metrics by test name if specified
        if test_name:
            filtered_metrics = [m for m in self.metrics if test_name in m.name]
        else:
            filtered_metrics = self.metrics
        
        if not filtered_metrics:
            self.logger.warning("No metrics to export")
            return
        
        # Convert to DataFrame and export
        df = pd.DataFrame([m.to_dict() for m in filtered_metrics])
        df.to_csv(output_file, index=False)
        
        self.logger.info(f"Metrics exported to: {output_file}")
    
    def get_trends(self, metric_name: str, window_size: int = 10) -> Dict[str, Any]:
        """Analyze trends for a specific metric"""
        metric_data = [m for m in self.metrics if m.name == metric_name]
        
        if not metric_data:
            return {"message": f"No data found for metric: {metric_name}"}
        
        # Sort by timestamp
        metric_data.sort(key=lambda x: x.timestamp)
        
        # Calculate moving average
        values = [m.value for m in metric_data]
        timestamps = [m.timestamp for m in metric_data]
        
        if len(values) < window_size:
            return {
                "metric": metric_name,
                "data_points": len(values),
                "values": values,
                "timestamps": timestamps
            }
        
        moving_avg = []
        for i in range(len(values) - window_size + 1):
            window_avg = np.mean(values[i:i + window_size])
            moving_avg.append(window_avg)
        
        return {
            "metric": metric_name,
            "data_points": len(values),
            "moving_average": moving_avg,
            "values": values,
            "timestamps": timestamps
        }
    
    def cleanup_old_metrics(self, days: int = 30):
        """Clean up metrics older than specified days"""
        cutoff_time = time.time() - (days * 24 * 60 * 60)
        
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute('DELETE FROM metrics WHERE timestamp < ?', (cutoff_time,))
                conn.commit()
                
                # Get count of deleted records
                cursor = conn.execute('SELECT changes()')
                deleted_count = cursor.fetchone()[0]
                
                self.logger.info(f"Cleaned up {deleted_count} old metrics")
                
        except Exception as e:
            self.logger.error(f"Failed to cleanup old metrics: {e}")


def create_test_metrics_collector(game_path: str) -> MetricsCollector:
    """Create a metrics collector configured for game testing"""
    collector = MetricsCollector()
    
    # Record initial system metrics
    system_metrics = collector.get_system_metrics()
    for name, value in system_metrics.items():
        collector.record_metric(f"initial_{name}", value)
    
    return collector


def main():
    """Main function for metrics collector"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Broken Divinity Metrics Collector")
    parser.add_argument("--game-path", help="Path to the game binary")
    parser.add_argument("--duration", type=int, default=5, help="Test duration in seconds")
    parser.add_argument("--output", help="Output file for metrics")
    parser.add_argument("--format", choices=["json", "csv"], default="json", help="Output format")
    parser.add_argument("--test-name", help="Specific test name to analyze")
    parser.add_argument("--cleanup", type=int, help="Clean up metrics older than N days")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    # Setup logging level
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        # Create metrics collector
        collector = create_test_metrics_collector(args.game_path)
        
        # Start system metrics collection
        collector.record_system_metrics(interval=0.5)
        
        # Run a test if game path is provided
        if args.game_path:
            from cli_wrapper import CLIWrapper
            
            # Record test start
            test_start_time = time.time()
            initial_memory = collector.get_system_metrics().get('memory_rss', 0)
            
            # Run the game
            wrapper = CLIWrapper(args.game_path)
            result = wrapper.run_headless(duration=args.duration)
            
            # Record test end
            test_end_time = time.time()
            test_duration = test_end_time - test_start_time
            final_memory = collector.get_system_metrics().get('memory_rss', 0)
            memory_usage = final_memory - initial_memory
            
            # Record test metrics
            collector.record_test_run(
                test_name="game_test",
                duration=test_duration,
                memory_usage=memory_usage,
                success=result.status.value == "completed",
                cpu_usage=collector.get_system_metrics().get('cpu_percent', 0)
            )
        
        # Generate report
        report = collector.generate_report(args.test_name)
        
        # Display results
        print("\nMetrics Report:")
        print("=" * 50)
        print(json.dumps(report, indent=2, default=str))
        
        # Export results if specified
        if args.output:
            if args.format == "json":
                collector.export_to_json(args.output, args.test_name)
            else:
                collector.export_to_csv(args.output, args.test_name)
            
            print(f"\nMetrics exported to: {args.output}")
        
        # Cleanup if specified
        if args.cleanup:
            collector.cleanup_old_metrics(args.cleanup)
            print(f"\nCleaned up metrics older than {args.cleanup} days")
        
        return 0
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    import sys
    sys.exit(main())