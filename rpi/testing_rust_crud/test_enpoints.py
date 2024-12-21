import numpy as np
import requests
import time

# base_url = "http://localhost:3000"
base_url = "http://192.168.2.39:3000"

def delete_range_of_locations(low:int, high:int) ->None:
    for i in range(low, high):
        response = requests.delete(f"{base_url}/location/{i}")
        # time.sleep(0.1)
        if response.status_code != 200:
            print(f"{response=}")
        

def create_n_locations(n:int) -> None:
    # Create
    for i in range(n):
        create_response = requests.post(f"{base_url}/location", json={"location": {"x":i, "y":1}})
        if create_response.status_code == 200:
            print("Created location:", create_response.json())
        # print("Create Response:", create_response)

def call_crud_endpoints(base_url):
    # # Create
    # create_response = requests.post(f"{base_url}/location", json={"name": "New Item"})
    # if create_response.status_code == 200:
    #     print("Create Response:", create_response.json())

    # Read
    read_response = requests.get(f"{base_url}/location")
    # if read_response.status_code == 200:
    print("Read Response:", read_response.json())

    # # Update
    # update_response = requests.put(f"{base_url}/update/1", json={"name": "Updated Item"})
    # if update_response.status_code == 200:
    #     print("Update Response:", update_response.json())

    # # Delete
    # delete_response = requests.delete(f"{base_url}/delete/1")
    # if delete_response.status_code == 200:
    #     print("Delete Response:", delete_response.json())

def create_animation() -> None:
    create_response = requests.post(f"{base_url}/frame_data", json={"frame_data": {"name":"Test for camera blanking", "speed":1}})
    if create_response.status_code != 200:
        print("ERROR:", create_response.json())
        # return 
    working_id: int= create_response.json().get("id", 1)
    print(f"{create_response.json()=}")
    # basics
    led_num = 250
    fade_amount = 20
    # setting the base color
    orange = (217,51,0)
    light_blue = (4,82,128)
    light_green = (2,92,26)
    working_color = light_green
    working_arr = [to_u32(*working_color)] * led_num
    red_lin = np.linspace(working_color[0],255,fade_amount).astype(int).tolist()
    green_lin = np.linspace(working_color[1],255,fade_amount).astype(int).tolist()
    blue_lin = np.linspace(working_color[2],255,fade_amount).astype(int).tolist()
    for index, (r,g,b) in enumerate(list(zip(red_lin, green_lin, blue_lin))):
        working_arr[index] = to_u32(r,g,b)
    working_arr = np.roll(working_arr, 20, axis=0).astype(int).tolist()
    
    for frame_id in range(1,led_num+1):
        frame_response = requests.post(f"{base_url}/frame", json={"frame": {"frame_id":frame_id,"parent_id":working_id,"data":str(working_arr)}})
        if frame_response.status_code != 200:
            print("ERROR:", create_response.json())
            # return 
        print(f"{frame_response.json()=}")
        working_arr = np.roll(working_arr, 1, axis=0).astype(int).tolist()

def from_u32(value:int) -> list:
    return [(value >> 16) & 0xFF, (value>>8) & 0xFF, value & 0xFF]

def to_u32(red, green, blue) -> int:
    return (red << 16) + (green << 8) + blue

def clear_near_camera() -> None:
    # get all of the frames from an animation
    animation_response = requests.get(f"{base_url}/animation/4")
    if animation_response.status_code != 200:
        print("ERROR:", animation_response.json())
    frames:dict = animation_response.json()
    # print(f"{frames}")
    for index,key in enumerate(frames.get("animation","").get("frames","")):
        print(f"index:{index}")
    
        working_frame = key
        frame_id = working_frame['id']
        working_id = working_frame['parent_id']
        working_arr = working_frame['data']
        for offset in range(21,28):
            working_arr[150-offset] = to_u32(0,0,0)
        working_url= f"{base_url}/frame/{frame_id}"
        print(f"{working_url=}")
        create_response = requests.put(working_url, json = {"frame":{"frame_id":frame_id,"parent_id":working_id,"data":str(working_arr)}})
        if create_response.status_code != 200:
            print("ERROR:", create_response.json())
            if input("press Y to continue: ").strip() != "Y":
                return



def main() -> None:
    # call_crud_endpoints(base_url=base_url)
    # create_n_locations(250)
    # delete_range_of_locations(1,279)
    # create_animation()
    clear_near_camera()

if __name__ == "__main__":
    main()