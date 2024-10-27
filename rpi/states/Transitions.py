from enum import Enum

class Transitions(Enum):
    to_house = 'to_house'
    to_street = 'to_street'
    stop = 'stop'
    animation_end = 'animation_end'
    timeout = 'timeout'
    unknown = 'unknown'