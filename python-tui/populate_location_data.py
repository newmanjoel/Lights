

import json
from pathlib import Path
from random import random
import sys
import numpy as np
import requests


led_num = 250
# distance_between_leds_in_mm = 100 # 10cm
side = 0

base_url = "http://192.168.2.24:3000"

# in cm [x,y]
# -> negative y
# V negative X
garden_left = [0,0]
garden_right = [-240,0]
door_right = [-240, -600]
door_left = [0, -600]

sections_x = [
    np.linspace(garden_left[0], garden_right[0], 32),
    np.linspace(garden_right[0], door_right[0], 88),
    np.linspace(door_right[0], door_left[0], 36),
    np.linspace(door_left[0], garden_left[0], 94)
]

sections_y = [
    np.linspace(garden_left[1], garden_right[1], 32),
    np.linspace(garden_right[1], door_right[1], 88),
    np.linspace(door_right[1], door_left[1], 36),
    np.linspace(door_left[1], garden_left[1], 94)
]

# DELETE the whole list!
# for index in reversed(range(350)):
#     result = requests.delete(f"{base_url}/location/{index}")
#     print(f"index:{index}:{result.json()=}")
# sys.exit(0)

all_points = {}
working_index = 0
for section_x, section_y in zip(sections_x,sections_y):
    for point in zip(section_x, section_y):
        # the random is used to make sure that each is a unique point
        x = float(point[0])
        y = float(point[1])
        print(f"settings point {x=} {y=}")
        working_point = {"x":x, "y":y}
        all_points[f"{working_index}"] = working_point
        working_index += 1
        # result = requests.post(f"{base_url}/location",data=json.dumps({"location":working_point}))
        # print(f"{result.json()=}")
        # sys.exit(1)

export_location = Path("exported_locations.json")
export_location.touch(exist_ok=True)
export_location.write_text(json.dumps(all_points, indent=4), encoding='utf-8')

print(f"Data saved at {export_location.absolute()}")

# last_x = 0
# last_y = 0
# for i in range(32): # side closest to the street
#     last_y = i*distance_between_leds_in_mm
#     requests.post(f"{base_url}/location",{"location":{"x":last_x, "y":last_y}})

# for i in range(88): # side closest to the garage
#     last_x = i*distance_between_leds_in_mm
#     requests.post(f"{base_url}/location",{"location":{"x":last_x, "y":last_y}})

# for i in range(36)