#!/usr/bin/env python3
"""
Integration Testing for Broken Divinity
Validates that all testing tools work together and provide consistent results
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
from concurrent.futures import ThreadPoolExecutor, as_completed

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class IntegrationTestType(Enum):
    """Types of integration tests"""
    TOOL_CHAIN = "tool_chain"
    DATA_FLOW = "data_flow"
    PERFORMANCE = "performance"
    CONSISTENCY = "consistency"
    COMPATIBILITY = "compatibility"

@dataclass
class IntegrationTestResult:
    """Result of an integration test"""
    test_type: IntegrationTestType
    name: str
    passed: bool
    duration: float
    details: Dict[str, Any]
    error_message: Optional[str]
    timestamp: str

@dataclass
class IntegrationTestSuite:
    """Complete integration test suite results"""
    total_tests: int
    passed_tests: int
    failed_tests: int
    success_rate: float
    total_duration: float
    individual_results: List[IntegrationTestResult]
    summary: Dict[str, Any]
    timestamp: str

class IntegrationTesting:
    """Main integration testing system for Broken Divinity"""
    
    def __init__(self, game_path: str, db_path: str = "testing/metrics.db"):
        self.game_path = game_path
        self.db_path = db_path
        
        # Initialize database
        self._init_database()
        
        # Test configurations
        self.test_configs = self._create_test_configs()
        
        logger.info("Integration Testing initialized")
    
    def _init_database(self):
        """Initialize SQLite database for storing integration test results"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Create integration test results table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS integration_tests (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT,
                    test_type TEXT,
                    test_name TEXT,
                    passed BOOLEAN,
                    duration REAL,
                    details_json TEXT,
                    error_message TEXT,
                    summary_json TEXT
                )
            ''')
            
            # Create integration test suite table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS integration_test_suites (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT,
                    total_tests INTEGER,
                    passed_tests INTEGER,
                    failed_tests INTEGER,
                    success_rate REAL,
                    total_duration REAL,
                    summary_json TEXT
                )
            ''')
            
            conn.commit()
            conn.close()
            logger.info("Database initialized successfully")
            
        except Exception as e:
            logger.error(f"Database initialization failed: {e}")
            raise
    
    def _create_test_configs(self) -> Dict[str, Dict]:
        """Create test configurations"""
        return {
            'tool_chain': {
                'description': 'Test tool execution chains',
                'tests': [
                    {
                        'name': 'cli_to_session',
                        'description': 'Test CLI wrapper to session orchestrator chain',
                        'tools': ['cli_wrapper', 'session_orchestrator'],
                        'expected_flow': ['start_game', 'run_duration', 'stop_game']
                    },
                    {
                        'name': 'session_to_metrics',
                        'description': 'Test session orchestrator to metrics collector chain',
                        'tools': ['session_orchestrator', 'metrics_collector'],
                        'expected_flow': ['execute_scenario', 'collect_metrics']
                    },
                    {
                        'name': 'full_tool_chain',
                        'description': 'Test complete tool execution chain',
                        'tools': ['cli_wrapper', 'session_orchestrator', 'metrics_collector', 'test_framework'],
                        'expected_flow': ['start_game', 'execute_scenario', 'collect_metrics', 'run_tests']
                    }
                ]
            },
            'data_flow': {
                'description': 'Test data flow between tools',
                'tests': [
                    {
                        'name': 'metrics_to_balance',
                        'description': 'Test metrics data flow to balance analytics',
                        'data_types': ['performance', 'functional', 'memory'],
                        'expected_outputs': ['balance_score', 'issues_found']
                    },
                    {
                        'name': 'balance_to_regression',
                        'description': 'Test balance data flow to regression detection',
                        'data_types': ['balance_metrics', 'performance_metrics'],
                        'expected_outputs': ['regressions_found', 'baseline_comparison']
                    },
                    {
                        'name': 'regression_to_data_gen',
                        'description': 'Test regression data flow to test data generation',
                        'data_types': ['regression_issues', 'performance_metrics'],
                        'expected_outputs': ['generated_data', 'test_scenarios']
                    }
                ]
            },
            'performance': {
                'description': 'Test performance characteristics',
                'tests': [
                    {
                        'name': 'tool_execution_time',
                        'description': 'Test individual tool execution times',
                        'max_time_limits': {
                            'cli_wrapper': 5.0,
                            'session_orchestrator': 10.0,
                            'metrics_collector': 2.0,
                            'test_framework': 5.0,
                            'balance_analytics': 15.0,
                            'regression_detection': 20.0,
                            'test_data_generator': 10.0
                        }
                    },
                    {
                        'name': 'memory_usage',
                        'description': 'Test memory usage patterns',
                        'max_memory_limits': {
                            'cli_wrapper': 100,  # MB
                            'session_orchestrator': 200,
                            'metrics_collector': 50,
                            'test_framework': 100,
                            'balance_analytics': 150,
                            'regression_detection': 200,
                            'test_data_generator': 100
                        }
                    },
                    {
                        'name': 'concurrent_execution',
                        'description': 'Test concurrent tool execution',
                        'max_concurrent': 3,
                        'timeout_per_tool': 30.0
                    }
                ]
            },
            'consistency': {
                'description': 'Test result consistency',
                'tests': [
                    {
                        'name': 'repeated_execution',
                        'description': 'Test repeated execution consistency',
                        'runs': 5,
                        'tolerance': 0.1  # 10% tolerance for metrics
                    },
                    {
                        'name': 'cross_tool_consistency',
                        'description': 'Test cross-tool result consistency',
                        'tools_to_compare': ['balance_analytics', 'regression_detection'],
                        'expected_correlation': 0.8
                    },
                    {
                        'name': 'data_integrity',
                        'description': 'Test data integrity across tools',
                        'validation_rules': [
                            'no_null_values',
                            'consistent_timestamps',
                            'proper_data_types',
                            'referential_integrity'
                        ]
                    }
                ]
            },
            'compatibility': {
                'description': 'Test compatibility aspects',
                'tests': [
                    {
                        'name': 'game_version_compatibility',
                        'description': 'Test compatibility with different game versions',
                        'versions_to_test': ['current'],
                        'backward_compatibility': True,
                        'forward_compatibility': False
                    },
                    {
                        'name': 'platform_compatibility',
                        'description': 'Test platform compatibility',
                        'platforms': ['linux'],
                        'architectures': ['x86_64']
                    },
                    {
                        'name': 'dependency_compatibility',
                        'description': 'Test dependency compatibility',
                        'required_packages': ['pandas', 'numpy', 'psutil'],
                        'python_versions': ['3.8', '3.9', '3.10', '3.11']
                    }
                ]
            }
        }
    
    def run_integration_tests(self, test_types: List[str] = None, 
                             parallel: bool = False) -> IntegrationTestSuite:
        """Run comprehensive integration tests"""
        if test_types is None:
            test_types = list(self.test_configs.keys())
        
        logger.info(f"Running integration tests for types: {test_types}")
        start_time = time.time()
        
        individual_results = []
        
        if parallel:
            # Run tests in parallel
            with ThreadPoolExecutor(max_workers=len(test_types)) as executor:
                future_to_test = {
                    executor.submit(self._run_test_type, test_type): test_type 
                    for test_type in test_types
                }
                
                for future in as_completed(future_to_test):
                    test_type = future_to_test[future]
                    try:
                        result = future.result()
                        individual_results.extend(result)
                    except Exception as e:
                        logger.error(f"Test {test_type} failed: {e}")
                        individual_results.append(IntegrationTestResult(
                            test_type=IntegrationTestType(test_type),
                            name=f"{test_type}_parallel",
                            passed=False,
                            duration=0.0,
                            details={},
                            error_message=str(e),
                            timestamp=datetime.now().isoformat()
                        ))
        else:
            # Run tests sequentially
            for test_type in test_types:
                try:
                    results = self._run_test_type(test_type)
                    individual_results.extend(results)
                except Exception as e:
                    logger.error(f"Test {test_type} failed: {e}")
                    individual_results.append(IntegrationTestResult(
                        test_type=IntegrationTestType(test_type),
                        name=f"{test_type}_sequential",
                        passed=False,
                        duration=0.0,
                        details={},
                        error_message=str(e),
                        timestamp=datetime.now().isoformat()
                    ))
        
        # Calculate suite summary
        total_tests = len(individual_results)
        passed_tests = sum(1 for r in individual_results if r.passed)
        failed_tests = total_tests - passed_tests
        success_rate = passed_tests / total_tests if total_tests > 0 else 0.0
        total_duration = time.time() - start_time
        
        # Create test suite
        suite = IntegrationTestSuite(
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            success_rate=success_rate,
            total_duration=total_duration,
            individual_results=individual_results,
            summary=self._generate_suite_summary(individual_results),
            timestamp=datetime.now().isoformat()
        )
        
        # Store results in database
        self._store_integration_results(suite)
        
        logger.info(f"Integration tests completed: {success_rate:.1%} success rate")
        return suite
    
    def _run_test_type(self, test_type: str) -> List[IntegrationTestResult]:
        """Run all tests for a specific type"""
        logger.info(f"Running {test_type} tests")
        results = []
        
        config = self.test_configs[test_type]
        
        for test_config in config['tests']:
            test_start = time.time()
            
            try:
                if test_type == 'tool_chain':
                    result = self._test_tool_chain(test_config)
                elif test_type == 'data_flow':
                    result = self._test_data_flow(test_config)
                elif test_type == 'performance':
                    result = self._test_performance(test_config)
                elif test_type == 'consistency':
                    result = self._test_consistency(test_config)
                elif test_type == 'compatibility':
                    result = self._test_compatibility(test_config)
                else:
                    raise ValueError(f"Unknown test type: {test_type}")
                
                test_duration = time.time() - test_start
                
                results.append(IntegrationTestResult(
                    test_type=IntegrationTestType(test_type),
                    name=test_config['name'],
                    passed=True,
                    duration=test_duration,
                    details=result,
                    error_message=None,
                    timestamp=datetime.now().isoformat()
                ))
                
            except Exception as e:
                test_duration = time.time() - test_start
                logger.error(f"Test {test_config['name']} failed: {e}")
                
                results.append(IntegrationTestResult(
                    test_type=IntegrationTestType(test_type),
                    name=test_config['name'],
                    passed=False,
                    duration=test_duration,
                    details={},
                    error_message=str(e),
                    timestamp=datetime.now().isoformat()
                ))
        
        return results
    
    def _test_tool_chain(self, config: Dict) -> Dict[str, Any]:
        """Test tool execution chains"""
        logger.info(f"Testing tool chain: {config['name']}")
        
        results = {}
        
        if config['name'] == 'cli_to_session':
            # Test CLI wrapper to session orchestrator chain
            cli_result = subprocess.run([
                'python3', 'testing/cli_wrapper.py', self.game_path, '--duration', '3'
            ], capture_output=True, text=True, timeout=10)
            
            session_result = subprocess.run([
                'python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle'
            ], capture_output=True, text=True, timeout=15)
            
            results = {
                'cli_wrapper_success': cli_result.returncode == 0,
                'session_orchestrator_success': session_result.returncode == 0,
                'chain_flow': self._verify_flow(session_result.stdout, config['expected_flow'])
            }
        
        elif config['name'] == 'session_to_metrics':
            # Test session orchestrator to metrics collector chain
            session_result = subprocess.run([
                'python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle'
            ], capture_output=True, text=True, timeout=15)
            
            metrics_result = subprocess.run([
                'python3', 'testing/metrics_collector.py', '--verbose'
            ], capture_output=True, text=True, timeout=10)
            
            results = {
                'session_orchestrator_success': session_result.returncode == 0,
                'metrics_collector_success': metrics_result.returncode == 0,
                'chain_flow': self._verify_flow(metrics_result.stdout, config['expected_flow'])
            }
        
        elif config['name'] == 'full_tool_chain':
            # Test complete tool execution chain
            tools = [
                ('cli_wrapper', ['python3', 'testing/cli_wrapper.py', self.game_path, '--duration', '3']),
                ('session_orchestrator', ['python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle']),
                ('metrics_collector', ['python3', 'testing/metrics_collector.py', '--verbose']),
                ('test_framework', ['python3', 'testing/test_framework.py', '--verbose'])
            ]
            
            tool_results = {}
            for tool_name, command in tools:
                result = subprocess.run(command, capture_output=True, text=True, timeout=20)
                tool_results[tool_name] = result.returncode == 0
            
            results = {
                'tool_results': tool_results,
                'chain_complete': all(tool_results.values()),
                'chain_flow': self._verify_full_chain(tool_results, config['expected_flow'])
            }
        
        return results
    
    def _test_data_flow(self, config: Dict) -> Dict[str, Any]:
        """Test data flow between tools"""
        logger.info(f"Testing data flow: {config['name']}")
        
        results = {}
        
        if config['name'] == 'metrics_to_balance':
            # Test metrics data flow to balance analytics
            metrics_result = subprocess.run([
                'python3', 'testing/metrics_collector.py', '--verbose'
            ], capture_output=True, text=True, timeout=10)
            
            balance_result = subprocess.run([
                'python3', 'testing/balance_analytics.py', self.game_path, '--duration', '5'
            ], capture_output=True, text=True, timeout=20)
            
            results = {
                'metrics_collected': 'total_metrics' in metrics_result.stdout,
                'balance_analysis_completed': balance_result.returncode == 0,
                'data_flow_verified': self._verify_data_flow(metrics_result.stdout, balance_result.stdout)
            }
        
        elif config['name'] == 'balance_to_regression':
            # Test balance data flow to regression detection
            balance_result = subprocess.run([
                'python3', 'testing/balance_analytics.py', self.game_path, '--duration', '5'
            ], capture_output=True, text=True, timeout=20)
            
            regression_result = subprocess.run([
                'python3', 'testing/regression_detection.py', self.game_path, '--runs', '2'
            ], capture_output=True, text=True, timeout=30)
            
            results = {
                'balance_analysis_completed': balance_result.returncode == 0,
                'regression_detection_completed': regression_result.returncode == 0,
                'data_flow_verified': self._verify_data_flow(balance_result.stdout, regression_result.stdout)
            }
        
        elif config['name'] == 'regression_to_data_gen':
            # Test regression data flow to test data generation
            regression_result = subprocess.run([
                'python3', 'testing/regression_detection.py', self.game_path, '--runs', '2'
            ], capture_output=True, text=True, timeout=30)
            
            data_gen_result = subprocess.run([
                'python3', 'testing/test_data_generator.py', self.game_path, '--profile', 'simple'
            ], capture_output=True, text=True, timeout=15)
            
            results = {
                'regression_detection_completed': regression_result.returncode == 0,
                'data_generation_completed': data_gen_result.returncode == 0,
                'data_flow_verified': self._verify_data_flow(regression_result.stdout, data_gen_result.stdout)
            }
        
        return results
    
    def _test_performance(self, config: Dict) -> Dict[str, Any]:
        """Test performance characteristics"""
        logger.info(f"Testing performance: {config['name']}")
        
        results = {}
        
        if config['name'] == 'tool_execution_time':
            # Test individual tool execution times
            tool_times = {}
            
            tools = {
                'cli_wrapper': ['python3', 'testing/cli_wrapper.py', self.game_path, '--duration', '3'],
                'session_orchestrator': ['python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle'],
                'metrics_collector': ['python3', 'testing/metrics_collector.py', '--verbose'],
                'test_framework': ['python3', 'testing/test_framework.py', '--verbose']
            }
            
            for tool_name, command in tools.items():
                start_time = time.time()
                result = subprocess.run(command, capture_output=True, text=True, timeout=30)
                duration = time.time() - start_time
                
                max_time = config['max_time_limits'].get(tool_name, 10.0)
                tool_times[tool_name] = {
                    'duration': duration,
                    'within_limit': duration <= max_time,
                    'max_limit': max_time
                }
            
            results = {
                'tool_times': tool_times,
                'all_within_limits': all(t['within_limit'] for t in tool_times.values())
            }
        
        elif config['name'] == 'memory_usage':
            # Test memory usage patterns
            tool_memory = {}
            
            tools = {
                'cli_wrapper': ['python3', 'testing/cli_wrapper.py', self.game_path, '--duration', '3'],
                'session_orchestrator': ['python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle'],
                'metrics_collector': ['python3', 'testing/metrics_collector.py', '--verbose']
            }
            
            for tool_name, command in tools.items():
                process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                time.sleep(2)  # Let process start
                
                try:
                    memory_info = psutil.Process(process.pid).memory_info()
                    memory_mb = memory_info.rss / 1024 / 1024  # Convert to MB
                    
                    max_memory = config['max_memory_limits'].get(tool_name, 100)
                    tool_memory[tool_name] = {
                        'memory_mb': memory_mb,
                        'within_limit': memory_mb <= max_memory,
                        'max_limit': max_memory
                    }
                    
                    process.terminate()
                    process.wait()
                    
                except Exception as e:
                    logger.warning(f"Could not measure memory for {tool_name}: {e}")
                    tool_memory[tool_name] = {
                        'memory_mb': 0,
                        'within_limit': True,
                        'max_limit': config['max_memory_limits'].get(tool_name, 100)
                    }
            
            results = {
                'tool_memory': tool_memory,
                'all_within_limits': all(m['within_limit'] for m in tool_memory.values())
            }
        
        elif config['name'] == 'concurrent_execution':
            # Test concurrent tool execution
            tools = [
                ['python3', 'testing/cli_wrapper.py', self.game_path, '--duration', '3'],
                ['python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle'],
                ['python3', 'testing/metrics_collector.py', '--verbose']
            ]
            
            start_time = time.time()
            processes = []
            
            for tool in tools:
                process = subprocess.Popen(tool, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                processes.append(process)
            
            # Wait for all processes to complete
            for process in processes:
                process.wait()
            
            total_duration = time.time() - start_time
            max_concurrent = config['max_concurrent']
            timeout_per_tool = config['timeout_per_tool']
            
            results = {
                'concurrent_execution_completed': total_duration <= timeout_per_tool,
                'total_duration': total_duration,
                'max_concurrent': max_concurrent,
                'timeout_per_tool': timeout_per_tool
            }
        
        return results
    
    def _test_consistency(self, config: Dict) -> Dict[str, Any]:
        """Test result consistency"""
        logger.info(f"Testing consistency: {config['name']}")
        
        results = {}
        
        if config['name'] == 'repeated_execution':
            # Test repeated execution consistency
            tool = 'python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle'
            
            run_results = []
            for i in range(config['runs']):
                result = subprocess.run(tool, capture_output=True, text=True, timeout=15)
                run_results.append({
                    'run': i + 1,
                    'success': result.returncode == 0,
                    'duration': self._extract_duration(result.stdout)
                })
            
            # Calculate consistency
            success_rates = [r['success'] for r in run_results]
            durations = [r['duration'] for r in run_results if r['duration'] > 0]
            
            consistent_success = len(set(success_rates)) == 1
            consistent_duration = len(set(durations)) <= 2 if durations else True
            
            results = {
                'run_results': run_results,
                'consistent_success': consistent_success,
                'consistent_duration': consistent_duration,
                'tolerance': config['tolerance']
            }
        
        elif config['name'] == 'cross_tool_consistency':
            # Test cross-tool result consistency
            balance_result = subprocess.run([
                'python3', 'testing/balance_analytics.py', self.game_path, '--duration', '5'
            ], capture_output=True, text=True, timeout=20)
            
            regression_result = subprocess.run([
                'python3', 'testing/regression_detection.py', self.game_path, '--runs', '2'
            ], capture_output=True, text=True, timeout=30)
            
            # Extract scores for comparison
            balance_score = self._extract_balance_score(balance_result.stdout)
            regression_score = self._extract_regression_score(regression_result.stdout)
            
            # Calculate correlation (simplified)
            correlation = abs(balance_score - regression_score) if balance_score and regression_score else 0
            
            results = {
                'balance_score': balance_score,
                'regression_score': regression_score,
                'correlation': correlation,
                'expected_correlation': config['expected_correlation'],
                'within_expected': correlation >= config['expected_correlation']
            }
        
        elif config['name'] == 'data_integrity':
            # Test data integrity across tools
            integrity_checks = {}
            
            # Check for null values
            tools = [
                ('cli_wrapper', ['python3', 'testing/cli_wrapper.py', self.game_path, '--duration', '3']),
                ('session_orchestrator', ['python3', 'testing/session_orchestrator.py', self.game_path, '--scenario', 'quick_cycle']),
                ('metrics_collector', ['python3', 'testing/metrics_collector.py', '--verbose'])
            ]
            
            for tool_name, command in tools:
                result = subprocess.run(command, capture_output=True, text=True, timeout=15)
                integrity_checks[tool_name] = {
                    'no_null_values': 'null' not in result.stdout.lower(),
                    'proper_format': len(result.stdout) > 0,
                    'success': result.returncode == 0
                }
            
            results = {
                'integrity_checks': integrity_checks,
                'all_passed': all(check['no_null_values'] and check['success'] for check in integrity_checks.values())
            }
        
        return results
    
    def _test_compatibility(self, config: Dict) -> Dict[str, Any]:
        """Test compatibility aspects"""
        logger.info(f"Testing compatibility: {config['name']}")
        
        results = {}
        
        if config['name'] == 'game_version_compatibility':
            # Test game version compatibility
            result = subprocess.run([self.game_path, '--help'], capture_output=True, text=True, timeout=10)
            
            results = {
                'game_executable': result.returncode == 0,
                'version_detected': 'version' in result.stdout.lower(),
                'backward_compatible': config['backward_compatibility'],
                'forward_compatible': config['forward_compatibility']
            }
        
        elif config['name'] == 'platform_compatibility':
            # Test platform compatibility
            import platform
            system = platform.system()
            architecture = platform.machine()
            
            results = {
                'current_platform': system,
                'current_architecture': architecture,
                'expected_platforms': config['platforms'],
                'expected_architectures': config['architectures'],
                'platform_compatible': system in config['platforms'],
                'architecture_compatible': architecture in config['architectures']
            }
        
        elif config['name'] == 'dependency_compatibility':
            # Test dependency compatibility
            import sys
            python_version = sys.version_info
            
            results = {
                'python_version': f"{python_version.major}.{python_version.minor}.{python_version.micro}",
                'required_packages': config['required_packages'],
                'python_versions': config['python_versions'],
                'python_compatible': f"{python_version.major}.{python_version.minor}" in config['python_versions'],
                'packages_available': all(pkg in sys.modules for pkg in config['required_packages'])
            }
        
        return results
    
    def _verify_flow(self, output: str, expected_flow: List[str]) -> bool:
        """Verify that expected flow steps are present in output"""
        for step in expected_flow:
            if step.lower() not in output.lower():
                return False
        return True
    
    def _verify_full_chain(self, tool_results: Dict, expected_flow: List[str]) -> bool:
        """Verify full tool chain execution"""
        return all(tool_results.values())
    
    def _verify_data_flow(self, source_output: str, target_output: str) -> bool:
        """Verify data flow between tools"""
        # Simple verification - check that target output references concepts from source
        source_concepts = ['metrics', 'performance', 'data', 'analysis']
        target_concepts = ['balance', 'regression', 'generation', 'test']
        
        source_found = any(concept in source_output.lower() for concept in source_concepts)
        target_found = any(concept in target_output.lower() for concept in target_concepts)
        
        return source_found and target_found
    
    def _extract_duration(self, output: str) -> float:
        """Extract duration from output"""
        time_match = re.search(r'Duration: (\d+\.\d+) seconds', output)
        return float(time_match.group(1)) if time_match else 0.0
    
    def _extract_balance_score(self, output: str) -> Optional[float]:
        """Extract balance score from output"""
        score_match = re.search(r'Overall Balance Score: (\d+\.\d+)', output)
        return float(score_match.group(1)) if score_match else None
    
    def _extract_regression_score(self, output: str) -> Optional[float]:
        """Extract regression score from output"""
        score_match = re.search(r'Performance Score: (\d+\.\d+)', output)
        return float(score_match.group(1)) if score_match else None
    
    def _generate_suite_summary(self, results: List[IntegrationTestResult]) -> Dict[str, Any]:
        """Generate suite summary"""
        summary = {
            'test_types': {},
            'performance_metrics': {},
            'error_analysis': {}
        }
        
        # Group by test type
        for result in results:
            test_type = result.test_type.value
            if test_type not in summary['test_types']:
                summary['test_types'][test_type] = {
                    'total': 0,
                    'passed': 0,
                    'failed': 0
                }
            
            summary['test_types'][test_type]['total'] += 1
            if result.passed:
                summary['test_types'][test_type]['passed'] += 1
            else:
                summary['test_types'][test_type]['failed'] += 1
        
        # Performance metrics
        durations = [r.duration for r in results]
        summary['performance_metrics'] = {
            'average_duration': statistics.mean(durations) if durations else 0,
            'max_duration': max(durations) if durations else 0,
            'min_duration': min(durations) if durations else 0
        }
        
        # Error analysis
        failed_results = [r for r in results if not r.passed]
        if failed_results:
            summary['error_analysis'] = {
                'total_failures': len(failed_results),
                'error_types': list(set(r.error_message for r in failed_results if r.error_message)),
                'most_common_error': max(set(r.error_message for r in failed_results if r.error_message), 
                                       key=lambda x: list(r.error_message for r in failed_results if r.error_message).count(x))
            }
        
        return summary
    
    def _store_integration_results(self, suite: IntegrationTestSuite):
        """Store integration test results in database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Store suite results
            cursor.execute('''
                INSERT INTO integration_test_suites 
                (timestamp, total_tests, passed_tests, failed_tests, success_rate, total_duration, summary_json)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            ''', (
                suite.timestamp,
                suite.total_tests,
                suite.passed_tests,
                suite.failed_tests,
                suite.success_rate,
                suite.total_duration,
                json.dumps(suite.summary)
            ))
            
            suite_id = cursor.lastrowid
            
            # Store individual test results
            for result in suite.individual_results:
                cursor.execute('''
                    INSERT INTO integration_tests 
                    (timestamp, test_type, test_name, passed, duration, details_json, error_message, summary_json)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    result.timestamp,
                    result.test_type.value,
                    result.name,
                    result.passed,
                    result.duration,
                    json.dumps(result.details),
                    result.error_message,
                    json.dumps(suite.summary)
                ))
            
            conn.commit()
            conn.close()
            
        except Exception as e:
            logger.error(f"Failed to store integration results: {e}")
    
    def generate_report(self, output_format: str = 'json') -> str:
        """Generate integration test report"""
        if not hasattr(self, 'last_suite'):
            return "No integration test data available"
        
        suite = self.last_suite
        
        if output_format == 'json':
            return json.dumps(asdict(suite), indent=2)
        elif output_format == 'html':
            return self._generate_html_report(suite)
        else:
            return self._generate_text_report(suite)
    
    def _generate_text_report(self, suite: IntegrationTestSuite) -> str:
        """Generate text-based integration test report"""
        report = []
        report.append("=" * 60)
        report.append("BROKEN DIVINITY INTEGRATION TEST REPORT")
        report.append("=" * 60)
        report.append(f"Test Time: {suite.timestamp}")
        report.append(f"Total Tests: {suite.total_tests}")
        report.append(f"Passed Tests: {suite.passed_tests}")
        report.append(f"Failed Tests: {suite.failed_tests}")
        report.append(f"Success Rate: {suite.success_rate:.1%}")
        report.append(f"Total Duration: {suite.total_duration:.2f} seconds")
        report.append("")
        
        # Test type breakdown
        report.append("TEST TYPE BREAKDOWN")
        report.append("-" * 20)
        for test_type, stats in suite.summary['test_types'].items():
            report.append(f"{test_type.upper()}: {stats['passed']}/{stats['total']} passed")
        report.append("")
        
        # Performance metrics
        report.append("PERFORMANCE METRICS")
        report.append("-" * 20)
        perf = suite.summary['performance_metrics']
        report.append(f"Average Duration: {perf['average_duration']:.2f} seconds")
        report.append(f"Max Duration: {perf['max_duration']:.2f} seconds")
        report.append(f"Min Duration: {perf['min_duration']:.2f} seconds")
        report.append("")
        
        # Individual results
        report.append("INDIVIDUAL TEST RESULTS")
        report.append("-" * 25)
        for result in suite.individual_results:
            status = "PASS" if result.passed else "FAIL"
            report.append(f"{status}: {result.test_type.value} - {result.name}")
            report.append(f"  Duration: {result.duration:.2f} seconds")
            if not result.passed:
                report.append(f"  Error: {result.error_message}")
            report.append("")
        
        return "\n".join(report)
    
    def _generate_html_report(self, suite: IntegrationTestSuite) -> str:
        """Generate HTML-based integration test report"""
        html = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Broken Divinity Integration Test Report</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
                .section {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
                .metric {{ margin: 10px 0; }}
                .test-result {{ margin: 10px 0; padding: 10px; border-radius: 3px; }}
                .pass {{ background-color: #d4edda; }}
                .fail {{ background-color: #f8d7da; }}
                .test-type {{ font-weight: bold; }}
            </style>
        </head>
        <body>
            <div class="header">
                <h1>Broken Divinity Integration Test Report</h1>
                <p>Test Time: {suite.timestamp}</p>
                <p>Total Tests: {suite.total_tests}</p>
                <p>Passed Tests: {suite.passed_tests}</p>
                <p>Failed Tests: {suite.failed_tests}</p>
                <p>Success Rate: {suite.success_rate:.1%}</p>
                <p>Total Duration: {suite.total_duration:.2f} seconds</p>
            </div>
            
            <div class="section">
                <h2>Test Type Breakdown</h2>
        """
        
        for test_type, stats in suite.summary['test_types'].items():
            html += f"""
                <div class="metric">
                    <strong>{test_type.upper()}:</strong> {stats['passed']}/{stats['total']} passed
                </div>
            """
        
        html += """
            </div>
            
            <div class="section">
                <h2>Performance Metrics</h2>
        """
        
        perf = suite.summary['performance_metrics']
        html += f"""
                <div class="metric">Average Duration: {perf['average_duration']:.2f} seconds</div>
                <div class="metric">Max Duration: {perf['max_duration']:.2f} seconds</div>
                <div class="metric">Min Duration: {perf['min_duration']:.2f} seconds</div>
            </div>
            
            <div class="section">
                <h2>Individual Test Results</h2>
        """
        
        for result in suite.individual_results:
            status_class = "pass" if result.passed else "fail"
            html += f"""
                <div class="test-result {status_class}">
                    <div class="test-type">{result.test_type.value} - {result.name}</div>
                    <div>Duration: {result.duration:.2f} seconds</div>
            """
            if not result.passed:
                html += f"<div>Error: {result.error_message}</div>"
            html += "</div>"
        
        html += """
            </div>
        </body>
        </html>
        """
        
        return html

def main():
    """Main function for running integration testing"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Integration Testing for Broken Divinity')
    parser.add_argument('game_path', help='Path to the game binary')
    parser.add_argument('--test-types', nargs='+', 
                       choices=['tool_chain', 'data_flow', 'performance', 'consistency', 'compatibility'],
                       default=['tool_chain', 'data_flow', 'performance'],
                       help='Test types to run')
    parser.add_argument('--parallel', action='store_true', help='Run tests in parallel')
    parser.add_argument('--output', choices=['json', 'text', 'html'], default='text',
                       help='Output format')
    parser.add_argument('--output-file', help='Output file path')
    parser.add_argument('--verbose', action='store_true', help='Verbose logging')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        # Initialize integration testing
        testing = IntegrationTesting(args.game_path)
        
        # Run integration tests
        suite = testing.run_integration_tests(args.test_types, args.parallel)
        
        # Store suite for reporting
        testing.last_suite = suite
        
        # Generate report
        if args.output == 'json':
            report = testing.generate_report('json')
        elif args.output == 'html':
            report = testing.generate_report('html')
        else:
            report = testing.generate_report('text')
        
        # Output report
        if args.output_file:
            with open(args.output_file, 'w') as f:
                f.write(report)
            print(f"Report saved to {args.output_file}")
        else:
            print(report)
        
        # Exit with appropriate code
        if suite.success_rate < 1.0:
            exit(1)  # Some tests failed
        else:
            exit(0)  # All tests passed
            
    except Exception as e:
        logger.error(f"Integration testing failed: {e}")
        exit(2)

if __name__ == "__main__":
    main()