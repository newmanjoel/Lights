import logging
from datetime import datetime, timedelta
import queue
import threading

import paho.mqtt.client as paho
from State import State
from Transitions import Transitions

# Add the root directory to the Python path
import os, sys

current_directory = os.path.dirname(os.path.abspath(__file__))
webservers_directory = os.path.abspath(os.path.join(current_directory, ".."))
sys.path.append(webservers_directory)

from common.common_objects import (
    setup_common_logger,
)

logger = logging.getLogger("idle-state")
logger = setup_common_logger(logger)

trigger_feedback = queue.Queue()
stop_event = threading.Event()
stop_event.clear()

mqtt_client = paho.Client()

def handle_networking(return_queue: queue.Queue, mqtt_client_local: paho.Client, stop_event:threading.Event): 
    def on_message(mosq, obj, msg):
        ll = logger.getChild('msg').getChild('<-')
        ll.info(f"{msg.topic} {msg.qos} {msg.payload}")
        match msg.topic:
            case 'trigger/stop':
                ll.debug(f"{msg.topic} {msg.qos} {msg.payload}")
                return_queue.put_nowait(Transitions.stop)
                stop_event.set()
            case _:
                ll.warning(f"Topic not assigned. {msg.topic} {msg.qos} {msg.payload}")
    mqtt_client_local.on_message = on_message
    mqtt_client_local.subscribe("zigbee2mqtt/motion_sensor_1", 0)
    mqtt_client_local.subscribe("trigger/stop", 0)
    mqtt_client_local.subscribe("#", 0)
    mqtt_client_local.connect('127.0.0.1', 1883, 60)
    logger.getChild('handle_networking').debug(f"mqtt_client should be set up. Event Status: {stop_event.is_set()}")

    while mqtt_client_local.loop() == 0:
        logger.getChild('mqtt_loop').debug('looping')
        if stop_event.is_set():
            return
        pass


# start up a thread and keep track of it
web_server_thread = threading.Thread(
        target=handle_networking,
        args=(trigger_feedback, mqtt_client, stop_event),
    )

def on_entry(state:State, what_happened:Transitions = Transitions.unknown) -> None:
    logger.getChild('on_entry').info(f"entering {state.name} due to {what_happened}")
    stop_event.clear()
    web_server_thread.start()

def on_exit(state, what_happened:Transitions = Transitions.unknown) -> None:
    logger.getChild('on_exit').debug(f"leaving {state.name} due to {what_happened}")
    stop_event.set()
    web_server_thread.join(timeout=1)

def subsribe_to_mqtt_event(state:State) -> Transitions | None:
    global trigger_feedback
    ll = logger.getChild('subscribe_to_mqtt_event')
    #ll.info(f"checking mqtt events")
    if trigger_feedback.qsize() == 0:
        return None
    result= trigger_feedback.get()
    if type(result) != Transitions:
        ll.error(f'received {result=} object which is not a transition object. Returning None')
        return None
    return result


def setup(state:State) -> State:
    state.on_entry_func = on_entry
    state.on_exit_func = on_exit
    state.triggering.append(subsribe_to_mqtt_event)
    return state
