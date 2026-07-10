#!/usr/bin/env python3
"""
Broken Divinity Test Framework
Base testing infrastructure for automated game testing
"""

import time
import logging
import traceback
from typing import Dict, List, Optional, Any, Type, Callable
from dataclasses import dataclass, field
from enum import Enum
from abc import ABC, abstractmethod
import json
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
import subprocess
import sys
import os


class TestStatus(Enum):
    """Test execution status"""
    PENDING = "pending"
    RUNNING = "running"
    PASSED = "passed"
    FAILED = "failed"
    SKIPPED = "skipped"
    ERROR = "error"


@dataclass
class TestResult:
    """Test execution result"""
    test_name: str
    status: TestStatus
    duration: float
    start_time: float
    end_time: float
    error_message: Optional[str] = None
    traceback: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            "test_name": self.test_name,
            "status": self.status.value,
            "duration": self.duration,
            "start_time": self.start_time,
            "end_time": self.end_time,
            "error_message": self.error_message,
            "traceback": self.traceback,
            "metadata": self.metadata
        }


class BaseTest(ABC):
    """Base test class that all tests should inherit from"""
    
    def __init__(self, name: str):
        self.name = name
        self.logger = logging.getLogger(f"{__name__}.{name}")
        self.setup_complete = False
        self.teardown_complete = False
    
    @abstractmethod
    def setup(self):
        """Setup test environment"""
        pass
    
    @abstractmethod
    def execute(self) -> TestResult:
        """Execute the test logic"""
        pass
    
    @abstractmethod
    def teardown(self):
        """Cleanup test environment"""
        pass
    
    def run(self) -> TestResult:
        """Execute test and return results"""
        start_time = time.time()
        self.setup_complete = False
        self.teardown_complete = False
        
        try:
            self.logger.info(f"Starting test: {self.name}")
            
            # Setup
            self.setup()
            self.setup_complete = True
            
            # Execute
            result = self.execute()
            
            # Teardown
            self.teardown()
            self.teardown_complete = True
            
            end_time = time.time()
            duration = end_time - start_time
            
            return TestResult(
                test_name=self.name,
                status=result.status,
                duration=duration,
                start_time=start_time,
                end_time=end_time,
                error_message=result.error_message,
                traceback=result.traceback,
                metadata=result.metadata
            )
            
        except Exception as e:
            end_time = time.time()
            duration = end_time - start_time
            
            self.logger.error(f"Test {self.name} failed with error: {e}")
            
            # Try to teardown even if test failed
            if self.setup_complete and not self.teardown_complete:
                try:
                    self.teardown()
                    self.teardown_complete = True
                except Exception as teardown_error:
                    self.logger.error(f"Teardown failed: {teardown_error}")
            
            return TestResult(
                test_name=self.name,
                status=TestStatus.ERROR,
                duration=duration,
                start_time=start_time,
                end_time=end_time,
                error_message=str(e),
                traceback=traceback.format_exc(),
                metadata={}
            )


class SimpleTest(BaseTest):
    """Simple test that executes a function"""
    
    def __init__(self, name: str, test_func: Callable[[], Any]):
        super().__init__(name)
        self.test_func = test_func
    
    def setup(self):
        """Setup for simple test"""
        pass
    
    def execute(self) -> TestResult:
        """Execute the test function"""
        try:
            self.test_func()
            return TestResult(
                test_name=self.name,
                status=TestStatus.PASSED,
                duration=0,
                start_time=time.time(),
                end_time=time.time(),
                metadata={}
            )
        except Exception as e:
            return TestResult(
                test_name=self.name,
                status=TestStatus.FAILED,
                duration=0,
                start_time=time.time(),
                end_time=time.time(),
                error_message=str(e),
                traceback=traceback.format_exc(),
                metadata={}
            )
    
    def teardown(self):
        """Teardown for simple test"""
        pass


class TestOrchestrator:
    """Test orchestrator for running multiple tests"""
    
    def __init__(self, max_workers: int = 4):
        self.tests: List[BaseTest] = []
        self.results: List[TestResult] = []
        self.max_workers = max_workers
        self.logger = logging.getLogger(__name__)
        
        # Setup logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
    
    def add_test(self, test: BaseTest):
        """Add a test to the orchestrator"""
        self.tests.append(test)
        self.logger.info(f"Added test: {test.name}")
    
    def add_simple_test(self, name: str, test_func: Callable[[], Any]):
        """Add a simple test function"""
        test = SimpleTest(name, test_func)
        self.add_test(test)
    
    def run_all(self, parallel: bool = False) -> Dict[str, TestResult]:
        """Run all tests"""
        self.logger.info(f"Running {len(self.tests)} tests")
        self.results = []
        
        if parallel and len(self.tests) > 1:
            return self._run_parallel()
        else:
            return self._run_sequential()
    
    def _run_sequential(self) -> Dict[str, TestResult]:
        """Run tests sequentially"""
        results = {}
        
        for test in self.tests:
            result = test.run()
            self.results.append(result)
            results[test.name] = result
            
            self.logger.info(f"Test {test.name}: {result.status.value}")
            if result.error_message:
                self.logger.error(f"Test {test.name} error: {result.error_message}")
        
        return results
    
    def _run_parallel(self) -> Dict[str, TestResult]:
        """Run tests in parallel"""
        results = {}
        
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            # Submit all tests
            future_to_test = {executor.submit(test.run): test for test in self.tests}
            
            # Collect results as they complete
            for future in as_completed(future_to_test):
                test = future_to_test[future]
                try:
                    result = future.result()
                    self.results.append(result)
                    results[test.name] = result
                    
                    self.logger.info(f"Test {test.name}: {result.status.value}")
                    if result.error_message:
                        self.logger.error(f"Test {test.name} error: {result.error_message}")
                        
                except Exception as e:
                    self.logger.error(f"Test {test.name} execution failed: {e}")
        
        return results
    
    def get_summary(self) -> Dict[str, Any]:
        """Get test execution summary"""
        if not self.results:
            return {"message": "No tests run yet"}
        
        total_tests = len(self.results)
        passed_tests = len([r for r in self.results if r.status == TestStatus.PASSED])
        failed_tests = len([r for r in self.results if r.status == TestStatus.FAILED])
        error_tests = len([r for r in self.results if r.status == TestStatus.ERROR])
        skipped_tests = len([r for r in self.results if r.status == TestStatus.SKIPPED])
        
        total_duration = sum(r.duration for r in self.results)
        avg_duration = total_duration / total_tests if total_tests > 0 else 0
        
        return {
            "total_tests": total_tests,
            "passed_tests": passed_tests,
            "failed_tests": failed_tests,
            "error_tests": error_tests,
            "skipped_tests": skipped_tests,
            "success_rate": passed_tests / total_tests if total_tests > 0 else 0,
            "total_duration": total_duration,
            "average_duration": avg_duration,
            "test_details": {r.test_name: r.status.value for r in self.results}
        }
    
    def export_results(self, output_file: str):
        """Export results to JSON file"""
        output_data = {
            "timestamp": time.time(),
            "summary": self.get_summary(),
            "detailed_results": [result.to_dict() for result in self.results]
        }
        
        with open(output_file, 'w') as f:
            json.dump(output_data, f, indent=2)
        
        self.logger.info(f"Results exported to: {output_file}")


def create_game_test(game_path: str, test_name: str, duration: int = 5) -> BaseTest:
    """Create a simple game test that runs the game for a specified duration"""
    
    def test_game():
        """Test function for game execution"""
        from cli_wrapper import CLIWrapper
        
        wrapper = CLIWrapper(game_path)
        result = wrapper.run_headless(duration=duration)
        
        if result.status.value == "failed":
            raise Exception(f"Game failed with exit code: {result.exit_code}")
    
    return SimpleTest(test_name, test_game)


def main():
    """Main function for test framework"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Broken Divinity Test Framework")
    parser.add_argument("--game-path", help="Path to the game binary")
    parser.add_argument("--duration", type=int, default=5, help="Test duration in seconds")
    parser.add_argument("--parallel", action="store_true", help="Run tests in parallel")
    parser.add_argument("--output", help="Output file for results")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    # Setup logging level
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        # Create test orchestrator
        orchestrator = TestOrchestrator(max_workers=4 if args.parallel else 1)
        
        # Add tests
        if args.game_path:
            # Add game tests
            orchestrator.add_test(create_game_test(args.game_path, "basic_startup", args.duration))
            orchestrator.add_test(create_game_test(args.game_path, "quick_cycle", args.duration))
            
            # Add some simple tests
            def math_test():
                assert 2 + 2 == 4
            orchestrator.add_simple_test("math_test", math_test)
            
            def string_test():
                assert "hello" == "hello"
            orchestrator.add_simple_test("string_test", string_test)
        
        # Run tests
        results = orchestrator.run_all(parallel=args.parallel)
        
        # Display results
        print("\nTest Results:")
        print("=" * 50)
        
        for test_name, result in results.items():
            print(f"\nTest: {test_name}")
            print(f"Status: {result.status.value}")
            print(f"Duration: {result.duration:.2f} seconds")
            
            if result.error_message:
                print(f"Error: {result.error_message}")
        
        # Display summary
        summary = orchestrator.get_summary()
        print("\nSummary:")
        print("=" * 50)
        print(f"Total Tests: {summary['total_tests']}")
        print(f"Passed: {summary['passed_tests']}")
        print(f"Failed: {summary['failed_tests']}")
        print(f"Errors: {summary['error_tests']}")
        print(f"Success Rate: {summary['success_rate']:.2%}")
        print(f"Total Duration: {summary['total_duration']:.2f} seconds")
        print(f"Average Duration: {summary['average_duration']:.2f} seconds")
        
        # Export results if specified
        if args.output:
            orchestrator.export_results(args.output)
            print(f"\nResults exported to: {args.output}")
        
        # Exit with appropriate code
        if summary['failed_tests'] > 0 or summary['error_tests'] > 0:
            return 1
        else:
            return 0
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())