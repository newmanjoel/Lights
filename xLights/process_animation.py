#!/usr/bin/env python3
"""
Convert a GIF animation to xLights-compatible format.
Each frame is processed, pixels are read, and an animation file is generated.
"""

import sys
from pathlib import Path
from PIL import Image
import requests


def load_gif_frames(gif_path: str) -> list[Image.Image]:
    """Load all frames from a GIF file."""
    img = Image.open(gif_path)
    img.seek(0)  # Go to first frame

    frames = []
    for i in range(img.n_frames):
        img.seek(i)
        frames.append(img.copy())

    return frames


def get_pixel_data(image: Image.Image) -> tuple[list[tuple[int, int, int]], int]:
    """
    Extract pixel data from an image.
    Returns: list of (x, y, color_as_hex) and the palette index mapping
    """
    pixels = []
    width, height = image.size

    for y in range(height):
        for x in range(width):
            # Get RGBA value
            r, g, b, a = image.getpixel((x, y))
            # Convert to hex color
            color_hex = f"#{r:02X}{g:02X}{b:02X}"
            pixels.append(((x, y), color_hex))

    return pixels


def to_u32(red, green, blue) -> int:
    return (red << 16) + (green << 8) + blue


def convert_frames_to_data(frames: list[Image.Image]) -> list[list[int]]:
    """
    Convert frames to a list of pixel data for sending to the controller.
    Each frame is represented as a list of (R, G, B) tuples.
    """
    frame_data = []
    for frame in frames:
        width, height = frame.size
        pixels = []
        for y in range(height):
            for x in range(width):
                pixel_value = frame.getpixel((x, y))
                if type(pixel_value) is tuple:
                    # handle this
                    r, g, b, _ = frame.getpixel((x, y))
                elif type(pixel_value) is int:
                    r,g,b = frame.getpalette()[3*pixel_value:3*pixel_value+3]

                pixels.append(to_u32(r, g, b))
                # pixels.append(pixel_value)
        frame_data.append(pixels)
    return frame_data

def send_data_to_controller(base_url:str, data: list[list[int]], animation_name: str, animation_speed: float = 15) -> None:
    create_response = requests.post(f"{base_url}/frame_data", json={"frame_data": {"name":f"{animation_name}", "speed":animation_speed}})
    if create_response.status_code != 200:
        print("ERROR:", create_response.json())
        return 
    working_id: int= create_response.json().get("id", 1)
    print(f"{create_response.json()=}")

    for frame_id, frame_pixels in enumerate(data, start=1):
        frame_response = requests.post(f"{base_url}/frame", json={"frame": {"frame_id":frame_id,"parent_id":working_id,"data":str(frame_pixels)}})
        if frame_response.status_code != 200:
            print("ERROR:", frame_response.json())
            # return 
        print(f"{frame_response.json()=}")
    print("Done sending frames!")
        



def main():

    gif_path = Path(r'/home/joel/GH/Lights/xLights/plasma.gif')
    name_of_animation = gif_path.stem
    base_url = "http://192.168.2.24:3000"

    # Load all frames from GIF
    frames = load_gif_frames(gif_path.absolute().__str__())
    data_to_send = convert_frames_to_data(frames)
    send_data_to_controller(base_url, data_to_send, name_of_animation, 30.3)

    print("Done!")


if __name__ == "__main__":
    main()
