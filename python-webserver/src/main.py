from dataclasses import dataclass
import threading
import time
import numpy as np
import pandas as pd
import requests
import streamlit as st
import altair as alt

st.write("Title")
base_url = 'http://192.168.2.24:3000'
col1, col2 = st.columns([1,1])
col1.write("col1")
col2.write("col2")


def get_current_data():
    current = requests.get(f"{base_url}/current")
    if current.status_code == 200:
        current_data = current.json()
        st.session_state["current"] = current_data
        st.session_state["animation_index"] = current_data.get("animation_index")
        st.session_state["current_index"] = current_data.get("frame_index")

def get_current_data_index():
    current = requests.get(f"{base_url}/current")
    if current.status_code == 200:
        current_data = current.json()
        st.session_state["current_index"] = current_data.get("frame_index")
        



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
    
    
    def to_hex(self):
        """Convert RGB values to hex color code."""
        hex_color = f"#{int(self.r):02X}{int(self.g):02X}{int(self.b):02X}"
        return hex_color


@dataclass
class FrameData:
    data: list[int]
    id: int
    frame_id: int
    parent_id: int
    
    @classmethod
    def from_dict(cls, data:dict):
        id = data.get("id", -1)
        frame_id = data.get("frame_id", -1)
        parent_id = data.get("parent_id", -1)
        data = data.get('data', [])
        return cls(id=id, frame_id=frame_id, parent_id=parent_id, data=data) # type: ignore

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

@st.cache_resource
def get_animation_at_index(index:int) -> Animation:
    current_animation = requests.get(f"{base_url}/animation/{index}")
    if current_animation.status_code == 200:
        current_animation_data = current_animation.json()
        assert("animation" in current_animation_data.keys())
        animation = Animation.from_dict(current_animation_data["animation"])
        return animation
    raise ValueError()
        

update_data_button = st.toggle("Update Data")
get_current_data()
live_updating_event = threading.Event()

if update_data_button:
    if 'current' in st.session_state.keys():
        get_current_data_index()
    # live_updating_event.clear()
    # threading.Thread(target=get_current_data_index, daemon=False, args=[live_updating_event]).start()
else:
    live_updating_event.set()
    


locations = requests.get(f'{base_url}/location')
if locations.status_code == 200:
    location_expander = st.expander("Locations")
    location_data = locations.json()
    df = pd.DataFrame.from_dict(location_data)
    df = df.set_index("id")

    st.session_state["location"] = df

    location_expander.write(df)


if "animation_index" in st.session_state.keys():
    animation = get_animation_at_index(st.session_state["animation_index"])
    if "current_animation" in st.session_state.keys():
        if animation != st.session_state['current_animation']:
            st.session_state['current_animation'] = animation
            st.write(f"Got the animation again at {time.time()}")
    else:
        st.session_state['current_animation'] = animation
    
    # current_frame_data = requests.get(f"{base_url}/")

chart_placeholder = st.empty()
timing_placeholder = st.empty()

if all(var in st.session_state.keys() for var in ["location", "current_index", "current_animation"]):
    start_time = time.time()
    current_frame_index = st.session_state['current_index']
    assert(current_frame_index is not None)
    current_frame:FrameData = st.session_state['current_animation'].frames[current_frame_index]
    working_df = st.session_state['location']
    int_colours = list(map(Color.from_int, current_frame.data))
    working_df["color"] = list(map(Color.to_hex, int_colours))
    chart = (alt.Chart(working_df).mark_square().encode(x="x:Q",y="y:Q", color=alt.Color("color:N", scale=None)).interactive())
    chart_placeholder.altair_chart(chart, use_container_width=True)
    end_time = time.time()
    timing_placeholder.write(f'It took {end_time-start_time:0.3f} seconds to render the graph')
    # data = np.ones(250)
    # x = st.session_state["location"].x
    # y = st.session_state["location"].y
    # st.scatter_chart(data, x=x, y=y)

time.sleep(0.2)
st.rerun()