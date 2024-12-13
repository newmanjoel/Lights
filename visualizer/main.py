import asyncio
import json
import pygame
import threading
from dataclasses import dataclass, field
from typing import List
import socket

# Data classes to represent animation and frames
@dataclass
class Animation:
    name: str
    speed: float
    id: int
    frames: List[List[int]]

@dataclass
class FrameData:
    frame_id: int

@dataclass
class Location:
    x: float
    y: float
    z: float = field(default=0)

# Initialize Pygame
pygame.init()
screen = pygame.display.set_mode((800, 600))
pygame.display.set_caption("3D Animation Viewer")
clock = pygame.time.Clock()
running = True

# Placeholder for animation data
animation = Animation(name="", speed=24.0, id=0, frames=[])
current_frame_id = 0
locations = [
    Location(10.0, 10.0),
    Location(20.0, 20.0),
    Location(30.0, 30.0),
    
]


# Function to update the frame from WebSocket data
async def listen_to_udp():
    global current_frame_id, running
    uri = "ws://localhost:12345"  # Change this to the WebSocket server address
    udp_ip = "localhost"  # Change this to the IP address to listen on
    udp_port = 12345       # Change this to the desired port

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((udp_ip, udp_port))
    sock.settimeout(1)

    while running:                                                                                                      
        try:                                                                                                         
            data, addr = sock.recvfrom(1024)  # Buffer size is 1024 bytes                                            
            try:                                                                                                     
                frame_data = json.loads(data.decode('utf-8'))                                                        
                # Process the JSON data                                                                              
                print(f"Received data from {addr}: {frame_data}")                                                     
                if "frame_id" in frame_data:
                    current_frame_id = frame_data["frame_id"]
                    print(f"Received frame ID: {current_frame_id}")
            except json.JSONDecodeError as e:                                                                        
                print(f"JSON decoding error: {e}")                                                                   
        except socket.error as e:                                                                                    
            print(f"Socket error: {e}")                                                                              
        except Exception as e:                                                                                       
            print(f"Unexpected error: {e}")         
        except socket.timeout:                                                                                   
                 print("Socket timed out, continuing to listen...")                                                                  
        finally:                                                                                                     
             # Optional: Add any cleanup code here                                                                    
            pass                     


# Function to draw points on the screen
def draw_frame(frame_id):
    screen.fill((0, 0, 0))  # Clear the screen with black
    if 0 <= frame_id < len(animation.frames):
        colors = animation.frames[frame_id]
        # print(f"{colors=}, {frame_id=}")
        for point, color in zip(locations, colors):
            # Assuming point is in the format [x, y, z]
            x, y, z = (point.x, point.y, point.z)
            r, g, b = color
            # Project z-axis by scaling the size of the circle
            size = max(1, 5 - (z / 50))  # Adjust the divisor for different depth scaling

            #                (surface, (color)        , center,           , radius, width)
            pygame.draw.circle(screen, (r, g, b), (x + 400, y + 300), int(size))

    pygame.display.flip()

# Function to load animation data
def load_animation():
    global animation
    # Example animation data, replace with actual loading logic
    animation_data = {
        "animation": {
            "name": "Example Animation",
            "speed": 24.0,
            "id": 1,
            "frames": [
                [(100, 100, 100), (200, 150, 100), (250, 200, 100)], 
                [(150, 100,100), (250, 150,100), (250, 200,100)]
            ]
        }
    }
    animation = Animation(**animation_data["animation"])
    print("Animation loaded:", animation)

def print_event_data(event: pygame.event):
    print(f"{event=}")

# Main loop for Pygame
def run_pygame():
    global running
    load_animation()
    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            if event.type == pygame.MOUSEBUTTONDOWN:
                print_event_data(event)
            elif event.type == pygame.MOUSEMOTION:
                print_event_data(event)
            elif event.type == pygame.MOUSEBUTTONUP:
                print_event_data(event)
            

        draw_frame(current_frame_id)
        clock.tick(animation.speed)

    pygame.quit()

# Start the WebSocket listener in a separate thread
websocket_thread = threading.Thread(target=lambda: asyncio.run(listen_to_udp()))
websocket_thread.start()

# Run the Pygame main loop
run_pygame()
