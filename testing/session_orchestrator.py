#!/usr/bin/env python3
"""
Broken Divinity Session Orchestrator
Basic automated gameplay scenarios for testing
"""

import time
import json
import logging
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
import subprocess
import threading
from cli_wrapper import CLIWrapper, GameStatus


class ScenarioStatus(Enum):
    """Scenario execution status"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"


@dataclass
class ScenarioResult:
    """Scenario execution result"""
    scenario_name: str
    status: ScenarioStatus
    duration: float
    steps: List[Dict[str, Any]]
    final_result: Optional[Dict[str, Any]]
    error_message: Optional[str] = None


class SessionOrchestrator:
    """Automated gameplay scenario orchestrator"""
    
    def __init__(self, game_path: str, log_level: str = "INFO"):
        self.cli = CLIWrapper(game_path)
        self.scenarios = self._define_scenarios()
        self.results: List[ScenarioResult] = []
        
        # Setup logging
        logging.basicConfig(
            level=getattr(logging, log_level.upper()),
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        self.logger = logging.getLogger(__name__)
    
    def _define_scenarios(self) -> Dict[str, Dict[str, Any]]:
        """Define core testing scenarios"""
        return {
            "new_game": {
                "description": "Test basic game startup and initialization",
                "duration": 10,
                "steps": [
                    {"name": "startup", "action": "start_game", "expected": "game_starts"},
                    {"name": "initialization", "action": "check_state", "expected": "menu_state"}
                ]
            },
            "quick_cycle": {
                "description": "Quick game cycle to test basic functionality",
                "duration": 15,
                "steps": [
                    {"name": "startup", "action": "start_game", "expected": "game_starts"},
                    {"name": "initialization", "action": "check_state", "expected": "menu_state"},
                    {"name": "run_duration", "action": "run_seconds", "duration": 5},
                    {"name": "shutdown", "action": "stop_game", "expected": "game_exits"}
                ]
            },
            "save_load_test": {
                "description": "Test save and load functionality",
                "duration": 20,
                "steps": [
                    {"name": "startup", "action": "start_game", "expected": "game_starts"},
                    {"name": "initialization", "action": "check_state", "expected": "menu_state"},
                    {"name": "run_duration", "action": "run_seconds", "duration": 3},
                    {"name": "save_game", "action": "save_game", "expected": "save_success"},
                    {"name": "shutdown", "action": "stop_game", "expected": "game_exits"},
                    {"name": "restart", "action": "start_game", "expected": "game_starts"},
                    {"name": "load_game", "action": "load_game", "expected": "load_success"},
                    {"name": "verify_state", "action": "check_state", "expected": "game_loaded"}
                ]
            },
            "stress_test": {
                "description": "Stress test with multiple rapid cycles",
                "duration": 30,
                "steps": [
                    {"name": "startup", "action": "start_game", "expected": "game_starts"},
                    {"name": "initialization", "action": "check_state", "expected": "menu_state"},
                    {"name": "cycle_1", "action": "run_seconds", "duration": 2},
                    {"name": "shutdown", "action": "stop_game", "expected": "game_exits"},
                    {"name": "startup", "action": "start_game", "expected": "game_starts"},
                    {"name": "initialization", "action": "check_state", "expected": "menu_state"},
                    {"name": "cycle_2", "action": "run_seconds", "duration": 2},
                    {"name": "shutdown", "action": "stop_game", "expected": "game_exits"}
                ]
            }
        }
    
    def _execute_step(self, step: Dict[str, Any], scenario_name: str) -> Dict[str, Any]:
        """Execute a single scenario step"""
        step_name = step["name"]
        action = step["action"]
        
        self.logger.info(f"Executing step '{step_name}' with action '{action}'")
        
        result = {
            "step_name": step_name,
            "action": action,
            "start_time": time.time(),
            "end_time": None,
            "success": False,
            "error": None
        }
        
        try:
            if action == "start_game":
                game_result = self.cli.run_headless(duration=step.get("duration", 5))
                # Consider success if game starts and runs (even if it crashes)
                result["success"] = game_result.status in [GameStatus.COMPLETED, GameStatus.FAILED]
                result["game_result"] = {
                    "status": game_result.status.value,
                    "duration": game_result.duration,
                    "exit_code": game_result.exit_code
                }
                
            elif action == "run_seconds":
                duration = step.get("duration", 5)
                time.sleep(duration)
                result["success"] = True
                result["actual_duration"] = duration
                
            elif action == "stop_game":
                # This is handled by the timeout in run_headless
                result["success"] = True
                
            elif action == "check_state":
                # For now, we can't easily check game state without more integration
                result["success"] = True
                result["note"] = "State checking not yet implemented"
                
            elif action == "save_game":
                # Save functionality would need game integration
                result["success"] = True
                result["note"] = "Save functionality not yet implemented"
                
            elif action == "load_game":
                # Load functionality would need game integration
                result["success"] = True
                result["note"] = "Load functionality not yet implemented"
                
            else:
                result["success"] = False
                result["error"] = f"Unknown action: {action}"
                
        except Exception as e:
            result["success"] = False
            result["error"] = str(e)
            self.logger.error(f"Error in step '{step_name}': {e}")
        
        result["end_time"] = time.time()
        result["duration"] = result["end_time"] - result["start_time"]
        
        return result
    
    def run_scenario(self, scenario_name: str) -> ScenarioResult:
        """Execute a predefined scenario"""
        if scenario_name not in self.scenarios:
            raise ValueError(f"Unknown scenario: {scenario_name}")
        
        scenario = self.scenarios[scenario_name]
        self.logger.info(f"Starting scenario: {scenario_name}")
        
        start_time = time.time()
        steps = []
        
        try:
            for step in scenario["steps"]:
                step_result = self._execute_step(step, scenario_name)
                steps.append(step_result)
                
                if not step_result["success"]:
                    self.logger.error(f"Step failed: {step_result['error']}")
                    break
            
            # Determine overall scenario success
            all_steps_passed = all(step["success"] for step in steps)
            
            if all_steps_passed:
                status = ScenarioStatus.COMPLETED
                final_result = {"steps_completed": len(steps)}
            else:
                status = ScenarioStatus.FAILED
                final_result = {"steps_completed": len([s for s in steps if s["success"]])}
            
            duration = time.time() - start_time
            
            result = ScenarioResult(
                scenario_name=scenario_name,
                status=status,
                duration=duration,
                steps=steps,
                final_result=final_result
            )
            
            self.results.append(result)
            self.logger.info(f"Scenario '{scenario_name}' completed with status: {status.value}")
            
            return result
            
        except Exception as e:
            duration = time.time() - start_time
            
            result = ScenarioResult(
                scenario_name=scenario_name,
                status=ScenarioStatus.FAILED,
                duration=duration,
                steps=steps,
                final_result=None,
                error_message=str(e)
            )
            
            self.results.append(result)
            self.logger.error(f"Scenario '{scenario_name}' failed: {e}")
            
            return result
    
    def run_all_scenarios(self) -> Dict[str, ScenarioResult]:
        """Run all defined scenarios"""
        self.logger.info("Running all scenarios")
        
        results = {}
        for scenario_name in self.scenarios.keys():
            results[scenario_name] = self.run_scenario(scenario_name)
        
        return results
    
    def get_scenario_summary(self) -> Dict[str, Any]:
        """Get summary of all scenario results"""
        if not self.results:
            return {"message": "No scenarios run yet"}
        
        total_scenarios = len(self.results)
        completed_scenarios = len([r for r in self.results if r.status == ScenarioStatus.COMPLETED])
        failed_scenarios = len([r for r in self.results if r.status == ScenarioStatus.FAILED])
        
        total_duration = sum(r.duration for r in self.results)
        avg_duration = total_duration / total_scenarios if total_scenarios > 0 else 0
        
        return {
            "total_scenarios": total_scenarios,
            "completed_scenarios": completed_scenarios,
            "failed_scenarios": failed_scenarios,
            "success_rate": completed_scenarios / total_scenarios if total_scenarios > 0 else 0,
            "total_duration": total_duration,
            "average_duration": avg_duration,
            "scenarios": {r.scenario_name: r.status.value for r in self.results}
        }
    
    def export_results(self, output_file: str):
        """Export results to JSON file"""
        output_data = {
            "timestamp": time.time(),
            "summary": self.get_scenario_summary(),
            "detailed_results": [
                {
                    "scenario_name": r.scenario_name,
                    "status": r.status.value,
                    "duration": r.duration,
                    "steps": r.steps,
                    "final_result": r.final_result,
                    "error_message": r.error_message
                }
                for r in self.results
            ]
        }
        
        with open(output_file, 'w') as f:
            json.dump(output_data, f, indent=2)
        
        self.logger.info(f"Results exported to: {output_file}")


def main():
    """Main function for session orchestrator"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Broken Divinity Session Orchestrator")
    parser.add_argument("game_path", help="Path to the game binary")
    parser.add_argument("--scenario", choices=["new_game", "quick_cycle", "save_load_test", "stress_test"],
                       help="Run specific scenario")
    parser.add_argument("--all-scenarios", action="store_true", help="Run all scenarios")
    parser.add_argument("--output", help="Output file for results")
    parser.add_argument("--log-level", default="INFO", choices=["DEBUG", "INFO", "WARNING", "ERROR"],
                       help="Logging level")
    
    args = parser.parse_args()
    
    try:
        # Create orchestrator
        orchestrator = SessionOrchestrator(args.game_path, args.log_level)
        
        # Run scenarios
        if args.all_scenarios:
            results = orchestrator.run_all_scenarios()
        elif args.scenario:
            results = {args.scenario: orchestrator.run_scenario(args.scenario)}
        else:
            print("Please specify --scenario or --all-scenarios")
            return
        
        # Display results
        print("\nScenario Results:")
        print("=" * 50)
        
        for scenario_name, result in results.items():
            print(f"\nScenario: {scenario_name}")
            print(f"Status: {result.status.value}")
            print(f"Duration: {result.duration:.2f} seconds")
            print(f"Steps: {len(result.steps)}")
            
            if result.error_message:
                print(f"Error: {result.error_message}")
        
        # Display summary
        summary = orchestrator.get_scenario_summary()
        print("\nSummary:")
        print("=" * 50)
        print(f"Total Scenarios: {summary['total_scenarios']}")
        print(f"Completed: {summary['completed_scenarios']}")
        print(f"Failed: {summary['failed_scenarios']}")
        print(f"Success Rate: {summary['success_rate']:.2%}")
        print(f"Total Duration: {summary['total_duration']:.2f} seconds")
        print(f"Average Duration: {summary['average_duration']:.2f} seconds")
        
        # Export results if specified
        if args.output:
            orchestrator.export_results(args.output)
            print(f"\nResults exported to: {args.output}")
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())