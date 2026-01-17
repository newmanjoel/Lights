import numpy as np
import pandas as pd
from serverobjects import Color, FrameData, Animation
from pathlib import Path
import json
import requests

base_url = "http://192.168.2.24:3000"

def get_locations(file_path:str) -> dict[str,dict[str,int]]:
    # load the x y z data
    exported_locations = Path(file_path)
    raw_file_contents = exported_locations.read_text(encoding='utf-8')
    # print(f"{raw_file_contents}")
    light_locations:dict = json.loads(raw_file_contents)
    return light_locations

def build_main_df(light_locations:dict, base_color:Color) -> pd.DataFrame:
    lights = pd.DataFrame([], columns=["index", "x", 'y', 'z', 'r','g', 'b'])
    for key,value in light_locations.items():
        # print(f"{key=} {value=}")
        new_row = {
            "index":key,
            "x":value["x"],
            "y":value["y"],
            "z": 0,
            "r": base_color.r,
            "g": base_color.g,
            "b": base_color.b,
        }
        lights.loc[len(lights)] = new_row # type: ignore

    print(f"{lights}")
    return lights


def main2():
    locations_path = Path(r'/home/joel/GH/Lights/common/locations.ods')
    locations = pd.read_excel(locations_path)
    # print(f"{locations}")
    base_color = Color(125,32,0) #dark orange
    lights = [base_color]*250
    # print(f"{lights}")

    # Define the solid box
    point_on_plane = (0,0,0)
    plane_normal_vector = (0,1,0)


def main():
    
    locations = get_locations(r"/home/joel/GH/Lights/python-webserver/src/exported_locations.json")
    base_color = Color(207, 95, 48)

    lights = build_main_df(locations, base_color)


    # want to fade in the x direction
    # x direction is to and from the door
    # y direction is to and from the garage wall

    sorted_by_x = lights.sort_values(by=['x'])
    print(f"{sorted_by_x}")

    min_x = min(lights["x"])
    max_x = max(lights["x"])
    bins = np.linspace(min_x, max_x, 50)
    # sorted_by_x["x_bin"] = pd.cut(x=sorted_by_x["x"], bins=100, labels=range(0,100), include_lowest=True)

    for working_x in range(int(min_x), int(max_x), 10):
        print(f"{working_x=}")
        valid_region = sorted_by_x['x'].between(working_x-5, working_x+5, "both")
        invalid_region = valid_region.__invert__()
        sorted_by_x.loc[valid_region, "z"] = "X"
        sorted_by_x.loc[invalid_region, "z"] = " "
        print(f"{sorted_by_x.to_string()}")
        # push this to an animation buffer
        result = requests.post(f"{base_url}/location",data=json.dumps({"location":working_point}))
        # print(f"{result.json()=}")



if __name__ == "__main__":
    main2()

