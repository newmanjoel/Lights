import logging
from datetime import datetime, timedelta

import paho.mqtt.client as paho
from Transitions import Transitions
from State import State
import threading
import queue

# Add the root directory to the Python path
import os, sys

current_directory = os.path.dirname(os.path.abspath(__file__))
webservers_directory = os.path.abspath(os.path.join(current_directory, ".."))
sys.path.append(webservers_directory)

from common.common_objects import (
    setup_common_logger,
)

logger = logging.getLogger("steady_on")
logger = setup_common_logger(logger)

state_timer = datetime.now()

trigger_feedback = queue.Queue()
stop_event = threading.Event()

mqtt_client = paho.Client()
mqtt_client.connect("127.0.0.1", 1883, 60)

def handle_networking(return_queue: queue.Queue, mqtt_client: paho.Client, stop_event:threading.Event):
    def on_message(mosq, obj, msg):
        ll = logging.getLogger('mqtt').getChild('msg').getChild('<-')
        match msg.topic:
            case 'trigger/stop':
                ll.debug(f"{msg.topic} {msg.qos} {msg.payload}")
                return_queue.put(Transitions.stop)
            case 'zigbee2mqtt/motion_sensor_1':
                ll.debug(f"{msg.topic} {msg.qos} {msg.payload}") 
                return_queue.put(Transitions.to_house)
            case _:
                ll.warning(f"Topic not assigned. {msg.topic} {msg.qos} {msg.payload}")
    mqtt_client.on_message = on_message
    mqtt_client.subscribe("zigbee2mqtt/motion_sensor_1/#", 0)
    mqtt_client.subscribe("trigger/stop", 0)

    while mqtt_client.loop() == 0:
        if stop_event.is_set():
            return
        pass

# start up a thread and keep track of it
web_server_thread = threading.Thread(
        target=handle_networking,
        args=(trigger_feedback, mqtt_client, stop_event),
    )

def on_entry(state:State, what_happened:Transitions = Transitions.unknown) -> None:
    global state_timer
    logger.getChild('on_entry').debug(f"entering {state.name} due to {what_happened}")
    state_timer = datetime.now() + timedelta(seconds=5)
    stop_event.clear()
    web_server_thread.start()

def on_exit(state, what_happened:Transitions = Transitions.unknown) -> None:
    logger.getChild('on_exit').debug(f"leaving {state.name} due to {what_happened}")
    stop_event.set()
    web_server_thread.join(timeout=1)

def state_timeout(state:State) -> Transitions | None:
    if datetime.now() > state_timer:
        return Transitions.timeout
    return None

def subsribe_to_mqtt_event(state:State) -> Transitions | None:
    global trigger_feedback
    if trigger_feedback.qsize() == 0:
        return None
    result= trigger_feedback.get()
    if type(result) != Transitions:
        logger.getChild('subscribe_to_mqtt').error(f'received {result=} object which is not a transition object. Returning None')
        return None
    return result
    

def setup(state:State) -> State:
    state.on_entry_func = on_entry
    state.on_exit_func = on_exit
    state.triggering.append(state_timeout)
    state.triggering.append(subsribe_to_mqtt_event)
    return state
