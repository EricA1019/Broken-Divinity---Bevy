#!/usr/bin/env python3
"""
Balance Analytics Engine for Broken Divinity
Analyzes game balance, combat mechanics, and progression systems
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
from datetime import datetime

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class BalanceIssueType(Enum):
    """Types of balance issues that can be detected"""
    OVERPOWERED = "overpowered"
    UNDERPOWERED = "underpowered"
    BROKEN_SYNERGY = "broken_synergy"
    PROGRESSION_ISSUE = "progression_issue"
    ECONOMIC_IMBALANCE = "economic_imbalance"
    COMBAT_IMBALANCE = "combat_imbalance"

@dataclass
class BalanceIssue:
    """Represents a balance issue found during analysis"""
    issue_type: BalanceIssueType
    severity: float  # 0.0 to 1.0
    description: str
    affected_systems: List[str]
    suggested_fix: str
    confidence: float  # 0.0 to 1.0
    metrics: Dict[str, Any]

@dataclass
class BalanceMetrics:
    """Comprehensive balance metrics for analysis"""
    combat_balance: Dict[str, float]
    progression_balance: Dict[str, float]
    economic_balance: Dict[str, float]
    overall_score: float
    issues_found: List[BalanceIssue]
    timestamp: str

class BalanceAnalyticsEngine:
    """Main balance analytics engine for Broken Divinity"""
    
    def __init__(self, game_path: str, db_path: str = "testing/metrics.db"):
        self.game_path = game_path
        self.db_path = db_path
        self.issues: List[BalanceIssue] = []
        self.metrics: Optional[BalanceMetrics] = None
        
        # Initialize database
        self._init_database()
        
        # Balance thresholds for analysis
        self.thresholds = {
            'combat_power_ratio': 1.5,  # Max ratio between strongest/weakest
            'progression_curve': 0.8,   # Minimum progression efficiency
            'economic_inflation': 2.0,  # Max inflation rate
            'synergy_threshold': 0.9,   # Minimum synergy effectiveness
            'survival_rate': 0.7,       # Minimum survival rate
        }
        
        logger.info("Balance Analytics Engine initialized")
    
    def _init_database(self):
        """Initialize SQLite database for storing balance analysis results"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Create balance analysis table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS balance_analysis (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT,
                    overall_score REAL,
                    combat_balance_score REAL,
                    progression_balance_score REAL,
                    economic_balance_score REAL,
                    issues_count INTEGER,
                    analysis_duration REAL,
                    metrics_json TEXT
                )
            ''')
            
            # Create balance issues table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS balance_issues (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    analysis_id INTEGER,
                    issue_type TEXT,
                    severity REAL,
                    description TEXT,
                    affected_systems TEXT,
                    suggested_fix TEXT,
                    confidence REAL,
                    metrics_json TEXT,
                    FOREIGN KEY (analysis_id) REFERENCES balance_analysis (id)
                )
            ''')
            
            conn.commit()
            conn.close()
            logger.info("Database initialized successfully")
            
        except Exception as e:
            logger.error(f"Database initialization failed: {e}")
            raise
    
    def run_analysis(self, duration: int = 10, scenarios: List[str] = None) -> BalanceMetrics:
        """Run comprehensive balance analysis"""
        if scenarios is None:
            scenarios = ['quick_cycle', 'new_game']
        
        logger.info(f"Starting balance analysis with scenarios: {scenarios}")
        start_time = time.time()
        
        try:
            # Step 1: Collect game data through scenarios
            game_data = self._collect_game_data(scenarios, duration)
            
            # Step 2: Analyze combat balance
            combat_balance = self._analyze_combat_balance(game_data)
            
            # Step 3: Analyze progression balance
            progression_balance = self._analyze_progression_balance(game_data)
            
            # Step 4: Analyze economic balance
            economic_balance = self._analyze_economic_balance(game_data)
            
            # Step 5: Detect balance issues
            self.issues = self._detect_balance_issues(
                combat_balance, progression_balance, economic_balance, game_data
            )
            
            # Step 6: Calculate overall balance score
            overall_score = self._calculate_overall_balance_score(
                combat_balance, progression_balance, economic_balance
            )
            
            # Step 7: Create balance metrics
            self.metrics = BalanceMetrics(
                combat_balance=combat_balance,
                progression_balance=progression_balance,
                economic_balance=economic_balance,
                overall_score=overall_score,
                issues_found=self.issues,
                timestamp=datetime.now().isoformat()
            )
            
            # Step 8: Store results in database
            analysis_duration = time.time() - start_time
            self._store_analysis_results(self.metrics, analysis_duration)
            
            logger.info(f"Balance analysis completed in {analysis_duration:.2f} seconds")
            logger.info(f"Overall balance score: {overall_score:.2f}")
            logger.info(f"Balance issues found: {len(self.issues)}")
            
            return self.metrics
            
        except Exception as e:
            logger.error(f"Balance analysis failed: {e}")
            raise
    
    def _collect_game_data(self, scenarios: List[str], duration: int) -> Dict[str, Any]:
        """Collect game data through automated scenarios"""
        logger.info("Collecting game data through scenarios")
        
        game_data = {
            'combat_stats': [],
            'progression_data': [],
            'economic_data': [],
            'survival_data': [],
            'timestamp': datetime.now().isoformat()
        }
        
        try:
            # Use session orchestrator to collect data
            for scenario in scenarios:
                logger.info(f"Running scenario: {scenario}")
                
                # Run scenario and collect metrics
                result = subprocess.run([
                    'python3', 'testing/session_orchestrator.py', 
                    self.game_path, '--scenario', scenario, '--log-level', 'DEBUG'
                ], capture_output=True, text=True, timeout=duration + 10)
                
                if result.returncode == 0:
                    # Parse output for game data
                    scenario_data = self._parse_scenario_output(result.stdout)
                    game_data['combat_stats'].extend(scenario_data.get('combat_stats', []))
                    game_data['progression_data'].extend(scenario_data.get('progression_data', []))
                    game_data['economic_data'].extend(scenario_data.get('economic_data', []))
                    game_data['survival_data'].extend(scenario_data.get('survival_data', []))
                else:
                    logger.warning(f"Scenario {scenario} failed: {result.stderr}")
        
        except Exception as e:
            logger.error(f"Game data collection failed: {e}")
        
        return game_data
    
    def _parse_scenario_output(self, output: str) -> Dict[str, Any]:
        """Parse scenario output to extract game data"""
        data = {
            'combat_stats': [],
            'progression_data': [],
            'economic_data': [],
            'survival_data': []
        }
        
        # Extract combat statistics
        combat_matches = re.findall(r'Combat: (\w+) damage: (\d+)', output)
        for match in combat_matches:
            data['combat_stats'].append({
                'type': match[0],
                'damage': int(match[1]),
                'timestamp': datetime.now().isoformat()
            })
        
        # Extract progression data
        progression_matches = re.findall(r'Progression: (\w+) level: (\d+)', output)
        for match in progression_matches:
            data['progression_data'].append({
                'type': match[0],
                'level': int(match[1]),
                'timestamp': datetime.now().isoformat()
            })
        
        # Extract economic data
        economic_matches = re.findall(r'Economic: (\w+) amount: (\d+)', output)
        for match in economic_matches:
            data['economic_data'].append({
                'type': match[0],
                'amount': int(match[1]),
                'timestamp': datetime.now().isoformat()
            })
        
        # Extract survival data
        survival_matches = re.findall(r'Survival: (\w+) status: (\w+)', output)
        for match in survival_matches:
            data['survival_data'].append({
                'type': match[0],
                'status': match[1],
                'timestamp': datetime.now().isoformat()
            })
        
        return data
    
    def _analyze_combat_balance(self, game_data: Dict[str, Any]) -> Dict[str, float]:
        """Analyze combat balance and effectiveness"""
        logger.info("Analyzing combat balance")
        
        combat_stats = game_data.get('combat_stats', [])
        
        if not combat_stats:
            return {
                'balance_score': 0.0,
                'damage_variance': 0.0,
                'effectiveness': 0.0,
                'issues': ['No combat data available']
            }
        
        # Calculate damage statistics
        damages = [stat['damage'] for stat in combat_stats]
        avg_damage = statistics.mean(damages) if damages else 0
        damage_variance = statistics.variance(damages) if len(damages) > 1 else 0
        
        # Analyze damage type distribution
        damage_types = {}
        for stat in combat_stats:
            damage_type = stat['type']
            damage_types[damage_type] = damage_types.get(damage_type, 0) + stat['damage']
        
        # Calculate balance score based on damage distribution
        if len(damage_types) > 1:
            max_damage = max(damage_types.values())
            min_damage = min(damage_types.values())
            damage_ratio = max_damage / min_damage if min_damage > 0 else float('inf')
            
            # Balance score inversely related to damage ratio
            balance_score = max(0, 1 - (damage_ratio - 1) / (self.thresholds['combat_power_ratio'] - 1))
        else:
            balance_score = 0.5  # Single damage type
        
        # Calculate effectiveness based on average damage
        effectiveness = min(1.0, avg_damage / 100)  # Normalize to 0-1 scale
        
        return {
            'balance_score': balance_score,
            'damage_variance': damage_variance,
            'effectiveness': effectiveness,
            'damage_types': damage_types,
            'avg_damage': avg_damage,
            'issues': []
        }
    
    def _analyze_progression_balance(self, game_data: Dict[str, Any]) -> Dict[str, float]:
        """Analyze progression balance and curve"""
        logger.info("Analyzing progression balance")
        
        progression_data = game_data.get('progression_data', [])
        
        if not progression_data:
            return {
                'balance_score': 0.0,
                'curve_smoothness': 0.0,
                'progression_rate': 0.0,
                'issues': ['No progression data available']
            }
        
        # Extract progression levels
        levels = [stat['level'] for stat in progression_data]
        
        # Calculate progression rate
        if len(levels) > 1:
            progression_rate = (levels[-1] - levels[0]) / len(levels)
        else:
            progression_rate = 0
        
        # Calculate curve smoothness (lower variance = smoother)
        level_variance = statistics.variance(levels) if len(levels) > 1 else 0
        curve_smoothness = max(0, 1 - level_variance / 10)  # Normalize to 0-1
        
        # Calculate balance score based on progression rate and smoothness
        balance_score = (progression_rate / 10 + curve_smoothness) / 2  # Normalize
        
        return {
            'balance_score': balance_score,
            'curve_smoothness': curve_smoothness,
            'progression_rate': progression_rate,
            'level_variance': level_variance,
            'issues': []
        }
    
    def _analyze_economic_balance(self, game_data: Dict[str, Any]) -> Dict[str, float]:
        """Analyze economic balance and inflation"""
        logger.info("Analyzing economic balance")
        
        economic_data = game_data.get('economic_data', [])
        
        if not economic_data:
            return {
                'balance_score': 0.0,
                'inflation_rate': 0.0,
                'resource_distribution': 0.0,
                'issues': ['No economic data available']
            }
        
        # Extract economic amounts
        amounts = [stat['amount'] for stat in economic_data]
        
        # Calculate inflation rate
        if len(amounts) > 1:
            inflation_rate = (amounts[-1] - amounts[0]) / amounts[0] if amounts[0] > 0 else 0
        else:
            inflation_rate = 0
        
        # Analyze resource distribution
        resource_types = {}
        for stat in economic_data:
            resource_type = stat['type']
            resource_types[resource_type] = resource_types.get(resource_type, 0) + stat['amount']
        
        # Calculate distribution balance
        if len(resource_types) > 1:
            max_resource = max(resource_types.values())
            min_resource = min(resource_types.values())
            resource_ratio = max_resource / min_resource if min_resource > 0 else float('inf')
            
            # Balance score inversely related to resource ratio
            distribution_balance = max(0, 1 - (resource_ratio - 1) / (self.thresholds['economic_inflation'] - 1))
        else:
            distribution_balance = 0.5  # Single resource type
        
        # Calculate balance score
        balance_score = (distribution_balance + max(0, 1 - inflation_rate / self.thresholds['economic_inflation'])) / 2
        
        return {
            'balance_score': balance_score,
            'inflation_rate': inflation_rate,
            'resource_distribution': distribution_balance,
            'resource_types': resource_types,
            'avg_amount': statistics.mean(amounts) if amounts else 0,
            'issues': []
        }
    
    def _detect_balance_issues(self, combat_balance: Dict, progression_balance: Dict, 
                              economic_balance: Dict, game_data: Dict) -> List[BalanceIssue]:
        """Detect balance issues based on analysis results"""
        logger.info("Detecting balance issues")
        
        issues = []
        
        # Check combat balance issues
        if combat_balance['balance_score'] < 0.5:
            issues.append(BalanceIssue(
                issue_type=BalanceIssueType.COMBAT_IMBALANCE,
                severity=1.0 - combat_balance['balance_score'],
                description="Combat system shows significant imbalance",
                affected_systems=['combat', 'damage', 'weapons'],
                suggested_fix="Review damage calculations and balance damage types",
                confidence=0.8,
                metrics=combat_balance
            ))
        
        # Check progression balance issues
        if progression_balance['balance_score'] < 0.5:
            issues.append(BalanceIssue(
                issue_type=BalanceIssueType.PROGRESSION_ISSUE,
                severity=1.0 - progression_balance['balance_score'],
                description="Progression curve is unbalanced",
                affected_systems=['progression', 'levels', 'experience'],
                suggested_fix="Adjust progression rates and curve smoothness",
                confidence=0.7,
                metrics=progression_balance
            ))
        
        # Check economic balance issues
        if economic_balance['balance_score'] < 0.5:
            issues.append(BalanceIssue(
                issue_type=BalanceIssueType.ECONOMIC_IMBALANCE,
                severity=1.0 - economic_balance['balance_score'],
                description="Economic system shows imbalance",
                affected_systems=['economy', 'resources', 'currency'],
                suggested_fix="Balance resource distribution and control inflation",
                confidence=0.8,
                metrics=economic_balance
            ))
        
        # Check for broken synergies
        if len(game_data.get('combat_stats', [])) > 0 and len(game_data.get('progression_data', [])) > 0:
            synergy_score = self._calculate_synergy_score(game_data)
            if synergy_score < self.thresholds['synergy_threshold']:
                issues.append(BalanceIssue(
                    issue_type=BalanceIssueType.BROKEN_SYNERGY,
                    severity=1.0 - synergy_score,
                    description="Combat and progression systems show poor synergy",
                    affected_systems=['combat', 'progression', 'synergy'],
                    suggested_fix="Improve integration between combat and progression systems",
                    confidence=0.6,
                    metrics={'synergy_score': synergy_score}
                ))
        
        return issues
    
    def _calculate_synergy_score(self, game_data: Dict) -> float:
        """Calculate synergy between different game systems"""
        # Simple synergy calculation based on data correlation
        combat_count = len(game_data.get('combat_stats', []))
        progression_count = len(game_data.get('progression_data', []))
        
        if combat_count == 0 or progression_count == 0:
            return 0.0
        
        # Calculate correlation (simplified)
        correlation = min(combat_count, progression_count) / max(combat_count, progression_count)
        return correlation
    
    def _calculate_overall_balance_score(self, combat_balance: Dict, 
                                       progression_balance: Dict, 
                                       economic_balance: Dict) -> float:
        """Calculate overall balance score"""
        scores = [
            combat_balance['balance_score'],
            progression_balance['balance_score'],
            economic_balance['balance_score']
        ]
        
        return statistics.mean(scores)
    
    def _store_analysis_results(self, metrics: BalanceMetrics, duration: float):
        """Store analysis results in database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Store main analysis results
            cursor.execute('''
                INSERT INTO balance_analysis 
                (timestamp, overall_score, combat_balance_score, progression_balance_score, 
                 economic_balance_score, issues_count, analysis_duration, metrics_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                metrics.timestamp,
                metrics.overall_score,
                metrics.combat_balance['balance_score'],
                metrics.progression_balance['balance_score'],
                metrics.economic_balance['balance_score'],
                len(metrics.issues_found),
                duration,
                json.dumps(asdict(metrics))
            ))
            
            analysis_id = cursor.lastrowid
            
            # Store individual issues
            for issue in metrics.issues_found:
                cursor.execute('''
                    INSERT INTO balance_issues 
                    (analysis_id, issue_type, severity, description, affected_systems, 
                     suggested_fix, confidence, metrics_json)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    analysis_id,
                    issue.issue_type.value,
                    issue.severity,
                    issue.description,
                    json.dumps(issue.affected_systems),
                    issue.suggested_fix,
                    issue.confidence,
                    json.dumps(asdict(issue))
                ))
            
            conn.commit()
            conn.close()
            
        except Exception as e:
            logger.error(f"Failed to store analysis results: {e}")
    
    def generate_report(self, output_format: str = 'json') -> str:
        """Generate balance analysis report"""
        if not self.metrics:
            return "No analysis data available"
        
        if output_format == 'json':
            return json.dumps(asdict(self.metrics), indent=2)
        elif output_format == 'html':
            return self._generate_html_report()
        else:
            return self._generate_text_report()
    
    def _generate_text_report(self) -> str:
        """Generate text-based balance report"""
        if not self.metrics:
            return "No analysis data available"
        
        report = []
        report.append("=" * 60)
        report.append("BROKEN DIVINITY BALANCE ANALYSIS REPORT")
        report.append("=" * 60)
        report.append(f"Analysis Time: {self.metrics.timestamp}")
        report.append(f"Overall Balance Score: {self.metrics.overall_score:.2f}")
        report.append("")
        
        # Combat Balance
        report.append("COMBAT BALANCE")
        report.append("-" * 20)
        combat = self.metrics.combat_balance
        report.append(f"Balance Score: {combat['balance_score']:.2f}")
        report.append(f"Average Damage: {combat['avg_damage']:.2f}")
        report.append(f"Damage Variance: {combat['damage_variance']:.2f}")
        report.append(f"Effectiveness: {combat['effectiveness']:.2f}")
        report.append("")
        
        # Progression Balance
        report.append("PROGRESSION BALANCE")
        report.append("-" * 25)
        progression = self.metrics.progression_balance
        report.append(f"Balance Score: {progression['balance_score']:.2f}")
        report.append(f"Progression Rate: {progression['progression_rate']:.2f}")
        report.append(f"Curve Smoothness: {progression['curve_smoothness']:.2f}")
        report.append("")
        
        # Economic Balance
        report.append("ECONOMIC BALANCE")
        report.append("-" * 20)
        economic = self.metrics.economic_balance
        report.append(f"Balance Score: {economic['balance_score']:.2f}")
        report.append(f"Inflation Rate: {economic['inflation_rate']:.2f}")
        report.append(f"Resource Distribution: {economic['resource_distribution']:.2f}")
        report.append("")
        
        # Issues
        report.append("BALANCE ISSUES")
        report.append("-" * 15)
        report.append(f"Total Issues Found: {len(self.metrics.issues_found)}")
        report.append("")
        
        for i, issue in enumerate(self.metrics.issues_found, 1):
            report.append(f"{i}. {issue.issue_type.value.upper()}")
            report.append(f"   Severity: {issue.severity:.2f}")
            report.append(f"   Description: {issue.description}")
            report.append(f"   Affected Systems: {', '.join(issue.affected_systems)}")
            report.append(f"   Suggested Fix: {issue.suggested_fix}")
            report.append(f"   Confidence: {issue.confidence:.2f}")
            report.append("")
        
        return "\n".join(report)
    
    def _generate_html_report(self) -> str:
        """Generate HTML-based balance report"""
        if not self.metrics:
            return "<html><body><h1>No analysis data available</h1></body></html>"
        
        html = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Broken Divinity Balance Analysis Report</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
                .section {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
                .metric {{ margin: 10px 0; }}
                .issue {{ background-color: #fff3cd; padding: 10px; margin: 10px 0; border-radius: 3px; }}
                .severity-high {{ color: #dc3545; }}
                .severity-medium {{ color: #fd7e14; }}
                .severity-low {{ color: #28a745; }}
            </style>
        </head>
        <body>
            <div class="header">
                <h1>Broken Divinity Balance Analysis Report</h1>
                <p>Analysis Time: {self.metrics.timestamp}</p>
                <p>Overall Balance Score: {self.metrics.overall_score:.2f}</p>
            </div>
            
            <div class="section">
                <h2>Combat Balance</h2>
                <div class="metric">Balance Score: {self.metrics.combat_balance['balance_score']:.2f}</div>
                <div class="metric">Average Damage: {self.metrics.combat_balance['avg_damage']:.2f}</div>
                <div class="metric">Damage Variance: {self.metrics.combat_balance['damage_variance']:.2f}</div>
                <div class="metric">Effectiveness: {self.metrics.combat_balance['effectiveness']:.2f}</div>
            </div>
            
            <div class="section">
                <h2>Progression Balance</h2>
                <div class="metric">Balance Score: {self.metrics.progression_balance['balance_score']:.2f}</div>
                <div class="metric">Progression Rate: {self.metrics.progression_balance['progression_rate']:.2f}</div>
                <div class="metric">Curve Smoothness: {self.metrics.progression_balance['curve_smoothness']:.2f}</div>
            </div>
            
            <div class="section">
                <h2>Economic Balance</h2>
                <div class="metric">Balance Score: {self.metrics.economic_balance['balance_score']:.2f}</div>
                <div class="metric">Inflation Rate: {self.metrics.economic_balance['inflation_rate']:.2f}</div>
                <div class="metric">Resource Distribution: {self.metrics.economic_balance['resource_distribution']:.2f}</div>
            </div>
            
            <div class="section">
                <h2>Balance Issues</h2>
                <p>Total Issues Found: {len(self.metrics.issues_found)}</p>
        """
        
        for issue in self.metrics.issues_found:
            severity_class = f"severity-{issue.severity:.2f}"
            html += f"""
                <div class="issue">
                    <h3>{issue.issue_type.value.upper()}</h3>
                    <p><strong>Severity:</strong> <span class="{severity_class}">{issue.severity:.2f}</span></p>
                    <p><strong>Description:</strong> {issue.description}</p>
                    <p><strong>Affected Systems:</strong> {', '.join(issue.affected_systems)}</p>
                    <p><strong>Suggested Fix:</strong> {issue.suggested_fix}</p>
                    <p><strong>Confidence:</strong> {issue.confidence:.2f}</p>
                </div>
            """
        
        html += """
            </div>
        </body>
        </html>
        """
        
        return html

def main():
    """Main function for running balance analytics"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Balance Analytics Engine for Broken Divinity')
    parser.add_argument('game_path', help='Path to the game binary')
    parser.add_argument('--duration', type=int, default=10, help='Analysis duration in seconds')
    parser.add_argument('--scenarios', nargs='+', default=['quick_cycle'], 
                       help='Scenarios to run for analysis')
    parser.add_argument('--output', choices=['json', 'text', 'html'], default='text',
                       help='Output format')
    parser.add_argument('--output-file', help='Output file path')
    parser.add_argument('--verbose', action='store_true', help='Verbose logging')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        # Initialize balance analytics engine
        engine = BalanceAnalyticsEngine(args.game_path)
        
        # Run analysis
        metrics = engine.run_analysis(args.duration, args.scenarios)
        
        # Generate report
        if args.output == 'json':
            report = engine.generate_report('json')
        elif args.output == 'html':
            report = engine.generate_report('html')
        else:
            report = engine.generate_report('text')
        
        # Output report
        if args.output_file:
            with open(args.output_file, 'w') as f:
                f.write(report)
            print(f"Report saved to {args.output_file}")
        else:
            print(report)
        
        # Exit with appropriate code
        if len(metrics.issues_found) > 0:
            exit(1)  # Issues found
        else:
            exit(0)  # No issues found
            
    except Exception as e:
        logger.error(f"Balance analysis failed: {e}")
        exit(2)

if __name__ == "__main__":
    main()