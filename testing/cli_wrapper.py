#!/usr/bin/env python3
"""
Broken Divinity CLI Wrapper
Python script to run game in headless mode for automated testing
"""

import subprocess
import time
import signal
import os
import sys
import json
from pathlib import Path
from typing import Dict, List, Optional, Union
from dataclasses import dataclass
from enum import Enum


class GameStatus(Enum):
    """Game execution status"""
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"


@dataclass
class GameResult:
    """Game execution result"""
    status: GameStatus
    duration: float
    exit_code: Optional[int]
    stdout: str
    stderr: str
    memory_usage: Optional[float] = None
    cpu_usage: Optional[float] = None


class CLIWrapper:
    """Python wrapper for Broken Divinity CLI"""
    
    def __init__(self, game_path: str):
        self.game_path = Path(game_path).absolute()
        self.process: Optional[subprocess.Popen] = None
        self.start_time: Optional[float] = None
        
        # Validate game path
        if not self.game_path.exists():
            raise FileNotFoundError(f"Game binary not found at: {self.game_path}")
        
        if not os.access(self.game_path, os.X_OK):
            raise PermissionError(f"Game binary is not executable: {self.game_path}")
    
    def run_headless(self, duration: int = 30, args: Optional[List[str]] = None) -> GameResult:
        """
        Run game in headless mode for specified duration
        
        Args:
            duration: Maximum runtime in seconds
            args: Additional CLI arguments to pass to the game
            
        Returns:
            GameResult containing execution information
        """
        if args is None:
            args = []
        
        # Build command
        command = [str(self.game_path), "--headless"] + args
        
        try:
            self.start_time = time.time()
            self.process = subprocess.Popen(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
                universal_newlines=True
            )
            
            # Wait for process to complete or timeout
            try:
                stdout, stderr = self.process.communicate(timeout=duration)
                exit_code = self.process.returncode
                duration_actual = time.time() - self.start_time
                
                if exit_code == 0:
                    status = GameStatus.COMPLETED
                else:
                    status = GameStatus.FAILED
                
                return GameResult(
                    status=status,
                    duration=duration_actual,
                    exit_code=exit_code,
                    stdout=stdout,
                    stderr=stderr
                )
                
            except subprocess.TimeoutExpired:
                # Terminate the process
                if self.process:
                    self.process.terminate()
                    try:
                        self.process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        self.process.kill()
                
                duration_actual = time.time() - self.start_time
                
                return GameResult(
                    status=GameStatus.TIMEOUT,
                    duration=duration_actual,
                    exit_code=None,
                    stdout="",
                    stderr="Game execution timed out"
                )
                
        except Exception as e:
            duration_actual = time.time() - self.start_time if self.start_time else 0
            
            return GameResult(
                status=GameStatus.FAILED,
                duration=duration_actual,
                exit_code=None,
                stdout="",
                stderr=str(e)
            )
    
    def run_scenario(self, scenario_name: str, scenario_config: Optional[Dict] = None) -> GameResult:
        """
        Run a predefined scenario
        
        Args:
            scenario_name: Name of the scenario to run
            scenario_config: Configuration for the scenario
            
        Returns:
            GameResult containing execution information
        """
        scenarios = {
            "new_game": {
                "duration": 10,
                "args": []
            },
            "quick_test": {
                "duration": 5,
                "args": []
            },
            "save_load_test": {
                "duration": 15,
                "args": []
            }
        }
        
        if scenario_name not in scenarios:
            raise ValueError(f"Unknown scenario: {scenario_name}")
        
        config = scenarios[scenario_name]
        if scenario_config:
            config.update(scenario_config)
        
        return self.run_headless(
            duration=config["duration"],
            args=config["args"]
        )
    
    def get_system_info(self) -> Dict:
        """Get system information"""
        return {
            "game_path": str(self.game_path),
            "game_exists": self.game_path.exists(),
            "game_executable": os.access(self.game_path, os.X_OK),
            "python_version": sys.version,
            "platform": sys.platform
        }
    
    def __del__(self):
        """Cleanup on destruction"""
        if self.process and self.process.poll() is None:
            self.process.terminate()


def main():
    """Main function for CLI wrapper"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Broken Divinity CLI Wrapper")
    parser.add_argument("game_path", help="Path to the game binary")
    parser.add_argument("--duration", type=int, default=30, help="Maximum runtime in seconds")
    parser.add_argument("--scenario", choices=["new_game", "quick_test", "save_load_test"], 
                       help="Run predefined scenario")
    parser.add_argument("--output", help="Output file for results")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")
    
    args = parser.parse_args()
    
    try:
        # Create wrapper
        wrapper = CLIWrapper(args.game_path)
        
        if args.verbose:
            print("System Info:")
            print(json.dumps(wrapper.get_system_info(), indent=2))
            print()
        
        # Run scenario or headless mode
        if args.scenario:
            result = wrapper.run_scenario(args.scenario)
        else:
            result = wrapper.run_headless(duration=args.duration)
        
        # Display results
        print(f"Status: {result.status.value}")
        print(f"Duration: {result.duration:.2f} seconds")
        if result.exit_code is not None:
            print(f"Exit Code: {result.exit_code}")
        
        if args.verbose:
            print("\nStdout:")
            print(result.stdout)
            print("\nStderr:")
            print(result.stderr)
        
        # Save results to file if specified
        if args.output:
            output_data = {
                "status": result.status.value,
                "duration": result.duration,
                "exit_code": result.exit_code,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "timestamp": time.time()
            }
            
            with open(args.output, 'w') as f:
                json.dump(output_data, f, indent=2)
            
            print(f"\nResults saved to: {args.output}")
        
        # Exit with appropriate code
        if result.status == GameStatus.COMPLETED:
            sys.exit(0)
        elif result.status == GameStatus.TIMEOUT:
            sys.exit(124)  # Standard timeout exit code
        else:
            sys.exit(1)
            
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()