from dataclasses import dataclass, field

import pandas as pd
import requests


@dataclass
class Color:
    r:int
    g:int
    b:int

    @classmethod
    def from_int(cls, value: int):
        r = (value >> 16) & 0xFF
        g = (value >> 8) & 0xFF
        b = value & 0xFF
        return cls(r,g,b)
    
    
    def to_hex(self) -> str:
        """Convert RGB values to hex color code."""
        hex_color = f"#{int(self.r):02X}{int(self.g):02X}{int(self.b):02X}"
        return hex_color
    
    def to_int(self) -> int:
        return (self.r << 16) | (self.g << 8) | self.b

@dataclass
class FrameData:
    data: list[int]
    id: int
    frame_id: int
    parent_id: int
    hex_colors: list[str] = field(default_factory=list, repr=False)
    
    @classmethod
    def from_dict(cls, data:dict):
        id = data.get("id", -1)
        frame_id = data.get("frame_id", -1)
        parent_id = data.get("parent_id", -1)
        data = data.get('data', [])
        return cls(id=id, frame_id=frame_id, parent_id=parent_id, data=data , hex_colors = list(map(Color.to_hex, map(Color.from_int, data)))) # type: ignore


@dataclass
class Animation:
    frames:list[FrameData]
    id:int
    name:str
    speed:int

    @classmethod
    def from_dict(cls, data:dict):
        id = data.get("id", -1)
        name = data.get("name", "no name found")
        speed = data.get("speed", -1)
        frames_data = data.get('frames', [])
        frames = list(map(FrameData.from_dict, frames_data))
        return cls(id=id, name=name, speed=speed, frames=frames)
    

def get_animation_at_index(base_url:str, index:int) -> Animation|None:
    current_animation = requests.get(f"{base_url}/animation/{index}")
    if current_animation.status_code == 200:
        current_animation_data = current_animation.json()
        assert("animation" in current_animation_data.keys())
        animation = Animation.from_dict(current_animation_data["animation"])
        return animation
    return None

def get_locations(base_url:str) -> pd.DataFrame|None:
    locations = requests.get(f'{base_url}/location')
    if locations.status_code == 200:
        location_data = locations.json()
        df = pd.DataFrame.from_dict(location_data)
        df = df.set_index("id")
        return df
    return None