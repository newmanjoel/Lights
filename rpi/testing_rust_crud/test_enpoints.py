import requests
import time

base_url = "http://localhost:3000"
# base_url = "http://192.168.2.39:3000"

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
    create_response = requests.post(f"{base_url}/frame_data", json={"frame_data": {"name":"animation_test", "speed":24}})
    if create_response.status_code != 200:
        print("ERROR:", create_response.json())
        # return 
    working_id: int= create_response.json().get("id", 1)
    print(f"{create_response.json()=}")
    working_arr = []
    for led_number in range(250):
        working_arr.append(to_u32(led_number,0,0))
    
    frame_response = requests.post(f"{base_url}/frame", json={"frame": {"frame_id":1,"parent_id":working_id,"data":str(working_arr)}})
    if frame_response.status_code != 200:
        print("ERROR:", create_response.json())
        # return 
    print(f"{frame_response.json()=}")

def from_u32(value:int) -> list:
    return [(value >> 16) & 0xFF, (value>>8) & 0xFF, value & 0xFF]

def to_u32(red, green, blue) -> int:
    return (red << 16) + (green << 8) + blue

def main() -> None:
    # call_crud_endpoints(base_url=base_url)
    # create_n_locations(250)
    # delete_range_of_locations(1,279)
    create_animation()

if __name__ == "__main__":
    main()