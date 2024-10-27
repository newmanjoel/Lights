from dataclasses import dataclass
from enum import Enum

class Direction(Enum):
    to_door = 1
    to_street = 2
    all_on = 3
    all_off = 4

def wave(size: int, direction:Direction) -> list:
    pass