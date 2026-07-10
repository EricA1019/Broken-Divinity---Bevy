#!/usr/bin/env python3
"""
Broken Divinity Testing Framework - Integration Script
Runs all testing tools and generates comprehensive reports
"""

import time
import json
import argparse
import logging
from pathlib import Path
from typing import Dict, Any, List

# Import our testing tools
from cli_wrapper import CLIWrapper, GameStatus
from session_orchestrator import SessionOrchestrator
from test_framework import TestOrchestrator, create_game_test
from metrics_collector import MetricsCollector, create_test_metrics_collector
from balance_analytics import BalanceAnalyticsEngine
from regression_detection import RegressionDetectionSystem
from test_data_generator import TestDataGenerator
from integration_testing import IntegrationTesting


def setup_logging(verbose: bool = False):
    """Setup logging configuration"""
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler('testing.log'),
            logging.StreamHandler()
        ]
    )


def run_comprehensive_test_suite(game_path: str, duration: int = 5) -> Dict[str, Any]:
    """Run comprehensive test suite with all tools"""
    results = {
        "timestamp": time.time(),
        "game_path": game_path,
        "duration": duration,
        "tools": {}
    }
    
    logger = logging.getLogger(__name__)
    logger.info("Starting comprehensive test suite")
    
    # 1. CLI Wrapper Test
    logger.info("Running CLI Wrapper test...")
    try:
        from cli_wrapper import CLIWrapper
        cli = CLIWrapper(game_path)
        cli_result = cli.run_headless(duration=duration)
        results["tools"]["cli_wrapper"] = {
            "status": cli_result.status.value,
            "duration": cli_result.duration,
            "exit_code": cli_result.exit_code
        }
        logger.info(f"CLI Wrapper: {cli_result.status.value}")
    except Exception as e:
        results["tools"]["cli_wrapper"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"CLI Wrapper failed: {e}")
    
    # 2. Session Orchestrator Test
    logger.info("Running Session Orchestrator test...")
    try:
        orchestrator = SessionOrchestrator(game_path, log_level="INFO")
        session_result = orchestrator.run_scenario("quick_cycle")
        results["tools"]["session_orchestrator"] = {
            "status": session_result.status.value,
            "duration": session_result.duration,
            "steps": len(session_result.steps)
        }
        logger.info(f"Session Orchestrator: {session_result.status.value}")
    except Exception as e:
        results["tools"]["session_orchestrator"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"Session Orchestrator failed: {e}")
    
    # 3. Test Framework Test
    logger.info("Running Test Framework test...")
    try:
        test_orchestrator = TestOrchestrator()
        test_orchestrator.add_test(create_game_test(game_path, "game_test", duration))
        test_orchestrator.add_test(create_game_test(game_path, "quick_test", duration))
        
        # Add some simple tests
        def math_test():
            assert 2 + 2 == 4
        test_orchestrator.add_simple_test("math_test", math_test)
        
        def string_test():
            assert "hello" == "hello"
        test_orchestrator.add_simple_test("string_test", string_test)
        
        test_results = test_orchestrator.run_all()
        results["tools"]["test_framework"] = {
            "total_tests": len(test_results),
            "passed_tests": len([r for r in test_results.values() if r.status.value == "passed"]),
            "failed_tests": len([r for r in test_results.values() if r.status.value == "failed"]),
            "success_rate": len([r for r in test_results.values() if r.status.value == "passed"]) / len(test_results) if test_results else 0
        }
        logger.info(f"Test Framework: {results['tools']['test_framework']['success_rate']:.2%} success rate")
    except Exception as e:
        results["tools"]["test_framework"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"Test Framework failed: {e}")
    
    # 4. Metrics Collector Test
    logger.info("Running Metrics Collector test...")
    try:
        metrics = create_test_metrics_collector(game_path)
        metrics.record_system_metrics(interval=0.5)
        
        # Run a test and collect metrics
        from cli_wrapper import CLIWrapper
        cli = CLIWrapper(game_path)
        result = cli.run_headless(duration=duration)
        
        # Record test metrics
        metrics.record_test_run(
            test_name="comprehensive_test",
            duration=result.duration,
            memory_usage=0,  # Would need to calculate this properly
            success=result.status.value == "completed"
        )
        
        # Generate report
        metrics_report = metrics.generate_report()
        results["tools"]["metrics_collector"] = metrics_report
        logger.info(f"Metrics Collector: {metrics_report['total_metrics']} metrics collected")
    except Exception as e:
        results["tools"]["metrics_collector"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"Metrics Collector failed: {e}")
    
    # 5. Balance Analytics Test
    logger.info("Running Balance Analytics test...")
    try:
        balance_engine = BalanceAnalyticsEngine(game_path)
        balance_result = balance_engine.run_analysis(duration=duration)
        results["tools"]["balance_analytics"] = {
            "balance_score": balance_result.get("overall_score", 0),
            "issues_found": len(balance_result.get("issues", [])),
            "duration": balance_result.get("duration", 0)
        }
        logger.info(f"Balance Analytics: Score {balance_result.get('overall_score', 0):.2f}")
    except Exception as e:
        results["tools"]["balance_analytics"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"Balance Analytics failed: {e}")
    
    # 6. Regression Detection Test
    logger.info("Running Regression Detection test...")
    try:
        regression_system = RegressionDetectionSystem(game_path)
        regression_result = regression_system.detect_regressions(runs=2)
        results["tools"]["regression_detection"] = {
            "regressions_found": len(regression_result.get("regressions", [])),
            "performance_score": regression_result.get("performance_score", 0),
            "duration": regression_result.get("duration", 0)
        }
        logger.info(f"Regression Detection: {len(regression_result.get('regressions', []))} regressions found")
    except Exception as e:
        results["tools"]["regression_detection"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"Regression Detection failed: {e}")
    
    # 7. Test Data Generator Test
    logger.info("Running Test Data Generator test...")
    try:
        data_generator = TestDataGenerator(game_path)
        data_result = data_generator.generate_data(profile_name="simple", scenario="general")
        results["tools"]["test_data_generator"] = {
            "data_generated": len(data_result.get("test_data", [])),
            "scenarios_created": len(data_result.get("scenarios", [])),
            "duration": data_result.get("duration", 0)
        }
        logger.info(f"Test Data Generator: {len(data_result.get('test_data', []))} data points generated")
    except Exception as e:
        results["tools"]["test_data_generator"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"Test Data Generator failed: {e}")
    
    # 8. Integration Testing
    logger.info("Running Integration Testing...")
    try:
        integration_testing = IntegrationTesting(game_path)
        integration_suite = integration_testing.run_integration_tests(
            test_types=['tool_chain', 'data_flow', 'performance'],
            parallel=True
        )
        results["tools"]["integration_testing"] = {
            "total_tests": integration_suite.total_tests,
            "passed_tests": integration_suite.passed_tests,
            "success_rate": integration_suite.success_rate,
            "duration": integration_suite.total_duration
        }
        logger.info(f"Integration Testing: {integration_suite.success_rate:.2%} success rate")
    except Exception as e:
        results["tools"]["integration_testing"] = {
            "status": "error",
            "error": str(e)
        }
        logger.error(f"Integration Testing failed: {e}")
    
    # 9. Generate Summary
    logger.info("Generating summary...")
    total_tools = len(results["tools"])
    successful_tools = len([t for t in results["tools"].values() if isinstance(t, dict) and (t.get("status") in ["completed", "passed"] or "success_rate" in t)])
    
    results["summary"] = {
        "total_tools": total_tools,
        "successful_tools": successful_tools,
        "success_rate": successful_tools / total_tools if total_tools > 0 else 0,
        "total_duration": sum(t.get("duration", 0) for t in results["tools"].values() if isinstance(t, dict) and "duration" in t)
    }
    
    logger.info(f"Test suite completed: {results['summary']['success_rate']:.2%} success rate")
    return results


def main():
    """Main function"""
    parser = argparse.ArgumentParser(description="Broken Divinity Comprehensive Testing Suite")
    parser.add_argument("game_path", help="Path to the game binary")
    parser.add_argument("--duration", type=int, default=5, help="Test duration in seconds")
    parser.add_argument("--output", help="Output file for results")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    # Setup logging
    setup_logging(args.verbose)
    
    logger = logging.getLogger(__name__)
    
    try:
        # Run comprehensive test suite
        results = run_comprehensive_test_suite(args.game_path, args.duration)
        
        # Display results
        print("\nComprehensive Test Suite Results:")
        print("=" * 50)
        
        for tool_name, tool_result in results["tools"].items():
            print(f"\n{tool_name.upper()}:")
            if isinstance(tool_result, dict):
                if "status" in tool_result:
                    print(f"  Status: {tool_result['status']}")
                if "duration" in tool_result:
                    print(f"  Duration: {tool_result['duration']:.2f} seconds")
                if "success_rate" in tool_result:
                    print(f"  Success Rate: {tool_result['success_rate']:.2%}")
                if "total_tests" in tool_result:
                    print(f"  Total Tests: {tool_result['total_tests']}")
                if "error" in tool_result:
                    print(f"  Error: {tool_result['error']}")
            else:
                print(f"  Result: {tool_result}")
        
        # Display summary
        summary = results["summary"]
        print(f"\nSummary:")
        print("=" * 50)
        print(f"Total Tools: {summary['total_tools']}")
        print(f"Successful Tools: {summary['successful_tools']}")
        print(f"Success Rate: {summary['success_rate']:.2%}")
        print(f"Total Duration: {summary['total_duration']:.2f} seconds")
        
        # Export results if specified
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(results, f, indent=2, default=str)
            print(f"\nResults exported to: {args.output}")
        
        # Exit with appropriate code
        if summary['success_rate'] < 0.5:
            return 1
        else:
            return 0
        
    except Exception as e:
        logger.error(f"Test suite failed: {e}")
        return 1


if __name__ == "__main__":
    import sys
    sys.exit(main())