#!/usr/bin/env python3
"""
Test Data Generator for Broken Divinity
Generates realistic test data for automated testing scenarios
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
import random
import string
from faker import Faker
import hashlib

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class DataType(Enum):
    """Types of test data that can be generated"""
    PLAYER_DATA = "player_data"
    COMBAT_DATA = "combat_data"
    PROGRESSION_DATA = "progression_data"
    ECONOMIC_DATA = "economic_data"
    WORLD_DATA = "world_data"
    SAVE_DATA = "save_data"
    SESSION_DATA = "session_data"

@dataclass
class GeneratedData:
    """Represents generated test data"""
    data_type: DataType
    scenario: str
    data: Dict[str, Any]
    metadata: Dict[str, Any]
    timestamp: str
    data_hash: str

@dataclass
class GenerationProfile:
    """Profile for data generation"""
    name: str
    description: str
    data_types: List[DataType]
    parameters: Dict[str, Any]
    complexity: str  # 'simple', 'medium', 'complex'
    target_count: int

class TestDataGenerator:
    """Main test data generator for Broken Divinity"""
    
    def __init__(self, game_path: str, output_dir: str = "testing/generated_data"):
        self.game_path = game_path
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        
        # Initialize Faker for realistic data
        self.faker = Faker()
        
        # Initialize database
        self._init_database()
        
        # Generation profiles
        self.profiles = self._create_generation_profiles()
        
        logger.info("Test Data Generator initialized")
    
    def _init_database(self):
        """Initialize SQLite database for storing generated data"""
        try:
            conn = sqlite3.connect("testing/metrics.db")
            cursor = conn.cursor()
            
            # Create generated data table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS generated_data (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT,
                    data_type TEXT,
                    scenario TEXT,
                    data_hash TEXT,
                    profile_name TEXT,
                    complexity TEXT,
                    data_json TEXT,
                    metadata_json TEXT
                )
            ''')
            
            # Create generation profiles table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS generation_profiles (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT UNIQUE,
                    description TEXT,
                    data_types TEXT,
                    parameters TEXT,
                    complexity TEXT,
                    target_count INTEGER
                )
            ''')
            
            conn.commit()
            conn.close()
            logger.info("Database initialized successfully")
            
        except Exception as e:
            logger.error(f"Database initialization failed: {e}")
            raise
    
    def _create_generation_profiles(self) -> Dict[str, GenerationProfile]:
        """Create predefined generation profiles"""
        profiles = {}
        
        # Simple profile
        profiles['simple'] = GenerationProfile(
            name='simple',
            description='Basic test data for quick validation',
            data_types=[DataType.PLAYER_DATA, DataType.SESSION_DATA],
            parameters={
                'player_count': 10,
                'session_count': 5,
                'complexity_factor': 0.5
            },
            complexity='simple',
            target_count=15
        )
        
        # Medium profile
        profiles['medium'] = GenerationProfile(
            name='medium',
            description='Comprehensive test data for thorough testing',
            data_types=[DataType.PLAYER_DATA, DataType.COMBAT_DATA, DataType.PROGRESSION_DATA, DataType.SESSION_DATA],
            parameters={
                'player_count': 50,
                'combat_count': 100,
                'progression_count': 30,
                'session_count': 20,
                'complexity_factor': 0.75
            },
            complexity='medium',
            target_count=200
        )
        
        # Complex profile
        profiles['complex'] = GenerationProfile(
            name='complex',
            description='Advanced test data for comprehensive testing',
            data_types=list(DataType),
            parameters={
                'player_count': 100,
                'combat_count': 500,
                'progression_count': 100,
                'economic_count': 200,
                'world_count': 50,
                'save_count': 30,
                'session_count': 50,
                'complexity_factor': 1.0
            },
            complexity='complex',
            target_count=1000
        )
        
        # Store profiles in database
        self._store_profiles(profiles)
        
        return profiles
    
    def _store_profiles(self, profiles: Dict[str, GenerationProfile]):
        """Store generation profiles in database"""
        try:
            conn = sqlite3.connect("testing/metrics.db")
            cursor = conn.cursor()
            
            for profile in profiles.values():
                cursor.execute('''
                    INSERT OR REPLACE INTO generation_profiles 
                    (name, description, data_types, parameters, complexity, target_count)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (
                    profile.name,
                    profile.description,
                    json.dumps([dt.value for dt in profile.data_types]),
                    json.dumps(profile.parameters),
                    profile.complexity,
                    profile.target_count
                ))
            
            conn.commit()
            conn.close()
            
        except Exception as e:
            logger.error(f"Failed to store profiles: {e}")
    
    def generate_data(self, profile_name: str, scenario: str = 'general') -> List[GeneratedData]:
        """Generate test data based on profile"""
        if profile_name not in self.profiles:
            raise ValueError(f"Profile '{profile_name}' not found")
        
        profile = self.profiles[profile_name]
        logger.info(f"Generating data for profile '{profile_name}' with scenario '{scenario}'")
        
        generated_data = []
        
        # Generate data for each type in profile
        for data_type in profile.data_types:
            logger.info(f"Generating {data_type.value} data")
            
            if data_type == DataType.PLAYER_DATA:
                data = self._generate_player_data(profile.parameters)
            elif data_type == DataType.COMBAT_DATA:
                data = self._generate_combat_data(profile.parameters)
            elif data_type == DataType.PROGRESSION_DATA:
                data = self._generate_progression_data(profile.parameters)
            elif data_type == DataType.ECONOMIC_DATA:
                data = self._generate_economic_data(profile.parameters)
            elif data_type == DataType.WORLD_DATA:
                data = self._generate_world_data(profile.parameters)
            elif data_type == DataType.SAVE_DATA:
                data = self._generate_save_data(profile.parameters)
            elif data_type == DataType.SESSION_DATA:
                data = self._generate_session_data(profile.parameters)
            else:
                continue
            
            # Create generated data object
            generated = GeneratedData(
                data_type=data_type,
                scenario=scenario,
                data=data,
                metadata={
                    'profile': profile_name,
                    'complexity': profile.complexity,
                    'generation_time': datetime.now().isoformat()
                },
                timestamp=datetime.now().isoformat(),
                data_hash=self._generate_hash(data)
            )
            
            generated_data.append(generated)
            
            # Store in database
            self._store_generated_data(generated)
        
        logger.info(f"Generated {len(generated_data)} data entries")
        return generated_data
    
    def _generate_player_data(self, params: Dict) -> Dict[str, Any]:
        """Generate realistic player data"""
        player_count = params.get('player_count', 10)
        
        players = []
        for i in range(player_count):
            player = {
                'id': f"player_{i:03d}",
                'name': self.faker.name(),
                'class': random.choice(['Warrior', 'Mage', 'Rogue', 'Cleric', 'Ranger']),
                'level': random.randint(1, 50),
                'experience': random.randint(0, 10000),
                'health': random.randint(50, 500),
                'max_health': random.randint(50, 500),
                'mana': random.randint(0, 200),
                'max_mana': random.randint(0, 200),
                'stamina': random.randint(0, 100),
                'max_stamina': random.randint(0, 100),
                'strength': random.randint(10, 100),
                'dexterity': random.randint(10, 100),
                'intelligence': random.randint(10, 100),
                'wisdom': random.randint(10, 100),
                'inventory': self._generate_inventory(),
                'equipment': self._generate_equipment(),
                'skills': self._generate_skills(),
                'achievements': self._generate_achievements(),
                'stats_history': self._generate_stats_history(),
                'created_at': self.faker.date_time_between('-30d', 'now').isoformat(),
                'last_login': self.faker.date_time_between('-7d', 'now').isoformat()
            }
            players.append(player)
        
        return {
            'players': players,
            'total_count': len(players),
            'generation_config': params
        }
    
    def _generate_combat_data(self, params: Dict) -> Dict[str, Any]:
        """Generate realistic combat data"""
        combat_count = params.get('combat_count', 100)
        
        combats = []
        for i in range(combat_count):
            combat = {
                'id': f"combat_{i:03d}",
                'timestamp': self.faker.date_time_between('-7d', 'now').isoformat(),
                'location': random.choice(['dungeon', 'overworld', 'colony', 'wilderness']),
                'participants': self._generate_combat_participants(),
                'actions': self._generate_combat_actions(),
                'results': self._generate_combat_results(),
                'duration': random.randint(10, 300),  # seconds
                'difficulty': random.choice(['easy', 'medium', 'hard', 'extreme']),
                'loot': self._generate_loot(),
                'experience_gained': random.randint(0, 1000),
                'notes': self.faker.sentence() if random.random() > 0.7 else None
            }
            combats.append(combat)
        
        return {
            'combats': combats,
            'total_count': len(combats),
            'generation_config': params
        }
    
    def _generate_progression_data(self, params: Dict) -> Dict[str, Any]:
        """Generate realistic progression data"""
        progression_count = params.get('progression_count', 30)
        
        progressions = []
        for i in range(progression_count):
            progression = {
                'id': f"progression_{i:03d}",
                'player_id': f"player_{random.randint(0, 99):03d}",
                'type': random.choice(['level', 'skill', 'achievement', 'milestone']),
                'name': self.faker.word(),
                'description': self.faker.sentence(),
                'level': random.randint(1, 50),
                'requirements': self._generate_requirements(),
                'rewards': self._generate_rewards(),
                'unlocked_at': self.faker.date_time_between('-30d', 'now').isoformat(),
                'progress_percentage': random.randint(0, 100),
                'is_completed': random.choice([True, False]),
                'related_achievements': self._generate_related_achievements(),
                'statistics': self._generate_progression_statistics()
            }
            progressions.append(progression)
        
        return {
            'progressions': progressions,
            'total_count': len(progression),
            'generation_config': params
        }
    
    def _generate_economic_data(self, params: Dict) -> Dict[str, Any]:
        """Generate realistic economic data"""
        economic_count = params.get('economic_count', 200)
        
        transactions = []
        for i in range(economic_count):
            transaction = {
                'id': f"transaction_{i:03d}",
                'timestamp': self.faker.date_time_between('-30d', 'now').isoformat(),
                'type': random.choice(['purchase', 'sale', 'trade', 'loot', 'reward']),
                'player_id': f"player_{random.randint(0, 99):03d}",
                'item': self._generate_item(),
                'quantity': random.randint(1, 100),
                'price': random.randint(1, 10000),
                'currency': random.choice(['gold', 'silver', 'copper', 'gems']),
                'location': random.choice(['market', 'dungeon', 'overworld', 'colony']),
                'vendor_id': f"vendor_{random.randint(0, 9):03d}" if random.random() > 0.5 else None,
                'notes': self.faker.sentence() if random.random() > 0.8 else None
            }
            transactions.append(transaction)
        
        return {
            'transactions': transactions,
            'total_count': len(transactions),
            'generation_config': params
        }
    
    def _generate_world_data(self, params: Dict) -> Dict[str, Any]:
        """Generate realistic world data"""
        world_count = params.get('world_count', 50)
        
        locations = []
        for i in range(world_count):
            location = {
                'id': f"location_{i:03d}",
                'name': self.faker.city(),
                'type': random.choice(['dungeon', 'settlement', 'wilderness', 'landmark']),
                'coordinates': {
                    'x': random.randint(-1000, 1000),
                    'y': random.randint(-1000, 1000)
                },
                'difficulty': random.choice(['easy', 'medium', 'hard', 'extreme']),
                'level_range': f"{random.randint(1, 10)}-{random.randint(11, 50)}",
                'features': self._generate_location_features(),
                'resources': self._generate_location_resources(),
                'threats': self._generate_location_threats(),
                'discovered': random.choice([True, False]),
                'visited_count': random.randint(0, 100),
                'last_visited': self.faker.date_time_between('-30d', 'now').isoformat() if random.random() > 0.5 else None
            }
            locations.append(location)
        
        return {
            'locations': locations,
            'total_count': len(locations),
            'generation_config': params
        }
    
    def _generate_save_data(self, params: Dict) -> Dict[str, Any]:
        """Generate realistic save data"""
        save_count = params.get('save_count', 30)
        
        saves = []
        for i in range(save_count):
            save = {
                'id': f"save_{i:03d}",
                'player_id': f"player_{random.randint(0, 99):03d}",
                'timestamp': self.faker.date_time_between('-30d', 'now').isoformat(),
                'slot_number': random.randint(1, 10),
                'game_version': f"1.{random.randint(0, 9)}.{random.randint(0, 9)}",
                'play_time': random.randint(0, 10000),  # seconds
                'current_location': random.choice(['dungeon', 'overworld', 'colony', 'menu']),
                'player_level': random.randint(1, 50),
                'current_health': random.randint(50, 500),
                'current_mana': random.randint(0, 200),
                'inventory_count': random.randint(10, 100),
                'quest_count': random.randint(0, 20),
                'achievement_count': random.randint(0, 50),
                'world_state': self._generate_world_state(),
                'save_notes': self.faker.sentence() if random.random() > 0.7 else None
            }
            saves.append(save)
        
        return {
            'saves': saves,
            'total_count': len(saves),
            'generation_config': params
        }
    
    def _generate_session_data(self, params: Dict) -> Dict[str, Any]:
        """Generate realistic session data"""
        session_count = params.get('session_count', 50)
        
        sessions = []
        for i in range(session_count):
            session = {
                'id': f"session_{i:03d}",
                'player_id': f"player_{random.randint(0, 99):03d}",
                'start_time': self.faker.date_time_between('-30d', 'now').isoformat(),
                'end_time': self.faker.date_time_between('-30d', 'now').isoformat(),
                'duration': random.randint(60, 7200),  # seconds
                'actions_count': random.randint(10, 1000),
                'combat_count': random.randint(0, 50),
                'location_changes': random.randint(0, 20),
                'items_collected': random.randint(0, 100),
                'experience_gained': random.randint(0, 5000),
                'death_count': random.randint(0, 10),
                'achievement_unlocked': random.choice([True, False]),
                'performance_metrics': self._generate_session_metrics(),
                'session_notes': self.faker.sentence() if random.random() > 0.8 else None
            }
            sessions.append(session)
        
        return {
            'sessions': sessions,
            'total_count': len(sessions),
            'generation_config': params
        }
    
    def _generate_inventory(self) -> List[Dict]:
        """Generate inventory items"""
        items = []
        for i in range(random.randint(5, 50)):
            item = {
                'id': f"item_{i:03d}",
                'name': self.faker.word(),
                'type': random.choice(['weapon', 'armor', 'consumable', 'material', 'quest']),
                'quantity': random.randint(1, 99),
                'quality': random.choice(['common', 'uncommon', 'rare', 'epic', 'legendary']),
                'value': random.randint(1, 10000),
                'weight': random.randint(1, 1000),
                'durability': random.randint(1, 100),
                'max_durability': random.randint(1, 100),
                'effects': self._generate_item_effects(),
                'description': self.faker.sentence()
            }
            items.append(item)
        return items
    
    def _generate_equipment(self) -> Dict:
        """Generate equipment"""
        return {
            'weapon': {
                'name': self.faker.word(),
                'type': random.choice(['sword', 'axe', 'bow', 'staff', 'dagger']),
                'damage': random.randint(10, 100),
                'durability': random.randint(50, 100),
                'max_durability': 100
            },
            'armor': {
                'name': self.faker.word(),
                'type': random.choice(['helmet', 'chest', 'gloves', 'boots', 'shield']),
                'defense': random.randint(5, 50),
                'durability': random.randint(50, 100),
                'max_durability': 100
            },
            'accessory': {
                'name': self.faker.word(),
                'type': random.choice(['ring', 'amulet', 'belt', 'cloak']),
                'effects': self._generate_item_effects()
            }
        }
    
    def _generate_skills(self) -> List[Dict]:
        """Generate skills"""
        skills = []
        for i in range(random.randint(3, 15)):
            skill = {
                'id': f"skill_{i:03d}",
                'name': self.faker.word(),
                'level': random.randint(1, 10),
                'max_level': 10,
                'experience': random.randint(0, 1000),
                'type': random.choice(['combat', 'crafting', 'social', 'exploration']),
                'effects': self._generate_skill_effects(),
                'unlocked': random.choice([True, False])
            }
            skills.append(skill)
        return skills
    
    def _generate_achievements(self) -> List[Dict]:
        """Generate achievements"""
        achievements = []
        for i in range(random.randint(0, 20)):
            achievement = {
                'id': f"achievement_{i:03d}",
                'name': self.faker.word(),
                'description': self.faker.sentence(),
                'rarity': random.choice(['common', 'uncommon', 'rare', 'epic', 'legendary']),
                'unlocked': random.choice([True, False]),
                'unlocked_at': self.faker.date_time_between('-30d', 'now').isoformat() if random.random() > 0.5 else None,
                'progress': random.randint(0, 100),
                'requirements': self._generate_requirements()
            }
            achievements.append(achievement)
        return achievements
    
    def _generate_stats_history(self) -> List[Dict]:
        """Generate stats history"""
        history = []
        for i in range(random.randint(5, 30)):
            entry = {
                'timestamp': self.faker.date_time_between('-30d', 'now').isoformat(),
                'health': random.randint(50, 500),
                'mana': random.randint(0, 200),
                'experience': random.randint(0, 10000),
                'level': random.randint(1, 50),
                'location': random.choice(['dungeon', 'overworld', 'colony', 'menu'])
            }
            history.append(entry)
        return history
    
    def _generate_combat_participants(self) -> List[Dict]:
        """Generate combat participants"""
        participants = []
        for i in range(random.randint(2, 8)):
            participant = {
                'id': f"participant_{i:03d}",
                'type': random.choice(['player', 'enemy', 'npc']),
                'name': self.faker.name(),
                'level': random.randint(1, 50),
                'health': random.randint(50, 500),
                'max_health': random.randint(50, 500),
                'is_alive': random.choice([True, False])
            }
            participants.append(participant)
        return participants
    
    def _generate_combat_actions(self) -> List[Dict]:
        """Generate combat actions"""
        actions = []
        for i in range(random.randint(5, 20)):
            action = {
                'timestamp': self.faker.date_time_between('-1h', 'now').isoformat(),
                'actor_id': f"participant_{random.randint(0, 7):03d}",
                'target_id': f"participant_{random.randint(0, 7):03d}",
                'type': random.choice(['attack', 'skill', 'item', 'defend', 'flee']),
                'damage_dealt': random.randint(0, 100),
                'damage_taken': random.randint(0, 100),
                'healing': random.randint(0, 50),
                'effects': self._generate_combat_effects()
            }
            actions.append(action)
        return actions
    
    def _generate_combat_results(self) -> Dict:
        """Generate combat results"""
        return {
            'victory': random.choice([True, False]),
            'duration': random.randint(10, 300),
            'experience_gained': random.randint(0, 1000),
            'loot': self._generate_loot(),
            'participants_alive': random.randint(1, 8),
            'participants_dead': random.randint(0, 7),
            'damage_dealt': random.randint(100, 5000),
            'damage_taken': random.randint(50, 2000),
            'healing_done': random.randint(0, 1000)
        }
    
    def _generate_loot(self) -> List[Dict]:
        """Generate loot"""
        loot = []
        for i in range(random.randint(0, 10)):
            item = {
                'name': self.faker.word(),
                'type': random.choice(['weapon', 'armor', 'consumable', 'material', 'currency']),
                'quantity': random.randint(1, 99),
                'quality': random.choice(['common', 'uncommon', 'rare', 'epic', 'legendary']),
                'value': random.randint(1, 10000)
            }
            loot.append(item)
        return loot
    
    def _generate_requirements(self) -> List[Dict]:
        """Generate requirements"""
        requirements = []
        for i in range(random.randint(1, 5)):
            requirement = {
                'type': random.choice(['level', 'skill', 'item', 'quest', 'stat']),
                'value': random.randint(1, 100),
                'current_value': random.randint(0, 100),
                'is_met': random.choice([True, False])
            }
            requirements.append(requirement)
        return requirements
    
    def _generate_rewards(self) -> List[Dict]:
        """Generate rewards"""
        rewards = []
        for i in range(random.randint(1, 5)):
            reward = {
                'type': random.choice(['experience', 'item', 'skill', 'currency', 'achievement']),
                'value': random.randint(1, 1000),
                'name': self.faker.word()
            }
            rewards.append(reward)
        return rewards
    
    def _generate_related_achievements(self) -> List[str]:
        """Generate related achievements"""
        return [f"achievement_{random.randint(0, 99):03d}" for _ in range(random.randint(0, 5))]
    
    def _generate_progression_statistics(self) -> Dict:
        """Generate progression statistics"""
        return {
            'time_to_complete': random.randint(60, 7200),  # seconds
            'attempts': random.randint(1, 20),
            'success_rate': random.uniform(0, 1),
            'average_progress': random.uniform(0, 1),
            'completion_streak': random.randint(0, 10)
        }
    
    def _generate_item(self) -> Dict:
        """Generate item"""
        return {
            'name': self.faker.word(),
            'type': random.choice(['weapon', 'armor', 'consumable', 'material', 'currency']),
            'quality': random.choice(['common', 'uncommon', 'rare', 'epic', 'legendary']),
            'value': random.randint(1, 10000),
            'weight': random.randint(1, 1000),
            'description': self.faker.sentence()
        }
    
    def _generate_location_features(self) -> List[str]:
        """Generate location features"""
        return random.sample([
            'dungeon', 'market', 'temple', 'tavern', 'blacksmith', 'alchemist',
            'garden', 'library', 'arena', 'cemetery', 'ruins', 'cave'
        ], random.randint(1, 5))
    
    def _generate_location_resources(self) -> List[str]:
        """Generate location resources"""
        return random.sample([
            'water', 'food', 'medicine', 'weapons', 'armor', 'materials',
            'gems', 'scrolls', 'potions', 'herbs', 'ores', 'wood'
        ], random.randint(1, 8))
    
    def _generate_location_threats(self) -> List[str]:
        """Generate location threats"""
        return random.sample([
            'bandits', 'monsters', 'traps', 'diseases', 'curses', 'magic',
            'wild animals', 'hostile npcs', 'environmental hazards'
        ], random.randint(0, 5))
    
    def _generate_world_state(self) -> Dict:
        """Generate world state"""
        return {
            'current_location': random.choice(['dungeon', 'overworld', 'colony', 'menu']),
            'time_of_day': random.choice(['dawn', 'day', 'dusk', 'night']),
            'weather': random.choice(['clear', 'rain', 'storm', 'fog', 'snow']),
            'quest_progress': {f"quest_{i:03d}": random.randint(0, 100) for i in range(random.randint(1, 5))},
            'faction_relations': {f"faction_{i:03d}": random.randint(-100, 100) for i in range(random.randint(1, 3))}
        }
    
    def _generate_session_metrics(self) -> Dict:
        """Generate session metrics"""
        return {
            'actions_per_minute': random.randint(5, 50),
            'combat_efficiency': random.uniform(0, 1),
            'survival_rate': random.uniform(0, 1),
            'exploration_coverage': random.uniform(0, 1),
            'resource_collection_rate': random.uniform(0, 1),
            'quest_completion_rate': random.uniform(0, 1)
        }
    
    def _generate_item_effects(self) -> List[str]:
        """Generate item effects"""
        return random.sample([
            'damage_boost', 'defense_boost', 'health_boost', 'mana_boost',
            'speed_boost', 'critical_chance', 'dodge_chance', 'experience_bonus'
        ], random.randint(0, 3))
    
    def _generate_skill_effects(self) -> List[str]:
        """Generate skill effects"""
        return random.sample([
            'damage_increase', 'healing', 'buff', 'debuff', 'cc', 'dot', 'hot'
        ], random.randint(0, 3))
    
    def _generate_combat_effects(self) -> List[str]:
        """Generate combat effects"""
        return random.sample([
            'stun', 'silence', 'slow', 'poison', 'burn', 'freeze', 'bleed'
        ], random.randint(0, 3))
    
    def _generate_hash(self, data: Dict) -> str:
        """Generate hash for data"""
        data_str = json.dumps(data, sort_keys=True)
        return hashlib.md5(data_str.encode()).hexdigest()
    
    def _store_generated_data(self, data: GeneratedData):
        """Store generated data in database"""
        try:
            conn = sqlite3.connect("testing/metrics.db")
            cursor = conn.cursor()
            
            cursor.execute('''
                INSERT INTO generated_data 
                (timestamp, data_type, scenario, data_hash, profile_name, complexity, data_json, metadata_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                data.timestamp,
                data.data_type.value,
                data.scenario,
                data.data_hash,
                data.metadata['profile'],
                data.metadata['complexity'],
                json.dumps(data.data),
                json.dumps(data.metadata)
            ))
            
            conn.commit()
            conn.close()
            
        except Exception as e:
            logger.error(f"Failed to store generated data: {e}")
    
    def get_profiles(self) -> Dict[str, GenerationProfile]:
        """Get available generation profiles"""
        return self.profiles
    
    def export_data(self, data: GeneratedData, format: str = 'json') -> str:
        """Export generated data in specified format"""
        if format == 'json':
            return json.dumps(asdict(data), indent=2)
        elif format == 'csv':
            return self._export_to_csv(data)
        elif format == 'xml':
            return self._export_to_xml(data)
        else:
            raise ValueError(f"Unsupported format: {format}")
    
    def _export_to_csv(self, data: GeneratedData) -> str:
        """Export data to CSV format"""
        # Simple CSV export for demonstration
        lines = []
        lines.append(f"Data Type,{data.data_type.value}")
        lines.append(f"Scenario,{data.scenario}")
        lines.append(f"Timestamp,{data.timestamp}")
        lines.append(f"Profile,{data.metadata['profile']}")
        lines.append(f"Complexity,{data.metadata['complexity']}")
        lines.append("")
        
        # Add data summary
        if data.data_type == DataType.PLAYER_DATA:
            lines.append("Player Data Summary")
            lines.append(f"Total Players,{len(data.data.get('players', []))}")
        elif data.data_type == DataType.COMBAT_DATA:
            lines.append("Combat Data Summary")
            lines.append(f"Total Combats,{len(data.data.get('combats', []))}")
        elif data.data_type == DataType.PROGRESSION_DATA:
            lines.append("Progression Data Summary")
            lines.append(f"Total Progressions,{len(data.data.get('progressions', []))}")
        
        return "\n".join(lines)
    
    def _export_to_xml(self, data: GeneratedData) -> str:
        """Export data to XML format"""
        xml = f'''<?xml version="1.0" encoding="UTF-8"?>
<generated_data>
    <data_type>{data.data_type.value}</data_type>
    <scenario>{data.scenario}</scenario>
    <timestamp>{data.timestamp}</timestamp>
    <metadata>
        <profile>{data.metadata['profile']}</profile>
        <complexity>{data.metadata['complexity']}</complexity>
    </metadata>
    <data>
        {json.dumps(data.data, indent=2)}
    </data>
</generated_data>'''
        return xml

def main():
    """Main function for running test data generation"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Test Data Generator for Broken Divinity')
    parser.add_argument('game_path', help='Path to the game binary')
    parser.add_argument('--profile', choices=['simple', 'medium', 'complex'], default='simple',
                       help='Generation profile to use')
    parser.add_argument('--scenario', default='general', help='Scenario name')
    parser.add_argument('--output-dir', default='testing/generated_data', 
                       help='Output directory for generated data')
    parser.add_argument('--export-format', choices=['json', 'csv', 'xml'], default='json',
                       help='Export format')
    parser.add_argument('--export-file', help='Export file path')
    parser.add_argument('--verbose', action='store_true', help='Verbose logging')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    try:
        # Initialize test data generator
        generator = TestDataGenerator(args.game_path, args.output_dir)
        
        # Generate data
        generated_data = generator.generate_data(args.profile, args.scenario)
        
        # Export data
        if args.export_file:
            # Export first data entry
            data_to_export = generated_data[0]
            exported_data = generator.export_data(data_to_export, args.export_format)
            
            with open(args.export_file, 'w') as f:
                f.write(exported_data)
            
            print(f"Data exported to {args.export_file}")
        else:
            print(f"Generated {len(generated_data)} data entries")
            for data in generated_data:
                print(f"- {data.data_type.value}: {len(data.data)} entries")
        
        exit(0)
            
    except Exception as e:
        logger.error(f"Data generation failed: {e}")
        exit(2)

if __name__ == "__main__":
    main()