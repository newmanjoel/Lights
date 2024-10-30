# used for being able to import stuff from other folders
import os
import sys

# Add the root directory to the Python path
current_directory = os.path.dirname(os.path.abspath(__file__))
rpi_directory = os.path.abspath(os.path.join(current_directory, ".."))
webservers_directory = os.path.abspath(os.path.join(rpi_directory, ".."))
sys.path.append(webservers_directory)

from common.common_objects import (
    setup_common_logger,
)
# from display import setup as setup_pixels

import logging

logger = logging.getLogger("state_machine")
logger = setup_common_logger(logger)

from dataclasses import dataclass
from typing import Callable, Self
from enum import Enum

try:
    import paho.mqtt.client as paho
except ImportError as e:
    logger.error("Could not import the mqtt library, please run `pip install paho-mqtt`")
    sys.exit(1)

# -----------------------------------------------------------------
import steady_on_state
import idle_state
from Transitions import Transitions
from State import State

# -----------------------------------------------------------------




def on_message(mosq, obj, msg):
    ll = logger.getChild('mqtt').getChild('msg').getChild('<-')
    match msg.topic:
        case 'trigger/stop':
            ll.debug(f"{msg.topic} {msg.qos} {msg.payload}")
        case _:
            ll.warning(f"Topic not assigned. {msg.topic} {msg.qos} {msg.payload}")



    print(f"{msg.topic} {msg.qos} {msg.payload}")
    mosq.publish('pong', 'ack', 0)

def on_publish(mosq, obj, mid):
    pass


# client = paho.Client()
# client.on_message = on_message
# client.on_publish = on_publish
# client.connect("127.0.0.1", 1883, 60)

# pixels = setup_pixels()



@dataclass
class StateMachine():
    current_state: State
    state_dict = dict()
    states = dict()
    '''
    {
        'idle': # <- from_state
            {
                'to_house': 'steady_on'
                'to_street': 'steady_on'
            }
    }
    '''

    def on_transition(self, from_state:State, to_state:State, when:Transitions) -> Self:
        '''Set up all of the possible transitions and when they occur.'''
        self.states[from_state.name] = from_state
        self.states[to_state.name] = to_state

        working_state: dict = self.state_dict.get(from_state.name, {})
        working_state[when] = to_state.name
        self.state_dict[from_state.name] = working_state
        # logger.getChild("on_transition").debug(f"{working_state=}")
        return self

    def loop(self) -> None:
        ll = logger.getChild('loop')
        # ll.debug('starting trigger_checking')
        result = self.current_state.check_triggers()
        if result is not None:
            ll.debug(f'{result=}')
        self.trigger(result)

    def trigger(self, what_trigger: Transitions) -> Self:
        '''Pass in all of the transitions, the dictionary will figure out if we need to change state or not'''
        if what_trigger is None:
            return self
        ll = logger.getChild('trigger')
        possible_states: dict = self.state_dict.get(self.current_state.name, {})
        if len(possible_states) == 0:
            return self
        if what_trigger in possible_states:
            new_state_name = possible_states[what_trigger]
            # ll.debug(f"want to change to state {new_state_name=} due to {what_trigger}")
            new_state = self.states[new_state_name]
            # ll.debug(f"{new_state=}")
            self.current_state.on_exit(what_trigger)
            # ll.debug(f'after on_exit')
            new_state.on_entry(what_trigger)
            # ll.debug(f'after on_entry')
            self.current_state = new_state
            logger.getChild('trigger').info(f'changed to state: {self.current_state.name}')
        return self


    

idle = State('idle')
steady_on = State('steady_on')
entry = State('entry')

states = StateMachine(entry)

steady_on = steady_on_state.setup(steady_on)
idle_state = idle_state.setup(idle)

# steady_on.on_entry_func = steady_on_state.on_entry
# steady_on.check_triggers = steady_on_state.trigger_generation

states.on_transition(entry, idle, Transitions.animation_end)
states.on_transition(idle, steady_on, Transitions.to_house)
states.on_transition(idle, idle, Transitions.timeout)

states.on_transition(idle, steady_on, Transitions.to_street)
states.on_transition(steady_on, idle, Transitions.stop)
states.on_transition(steady_on, idle, Transitions.timeout)


logger.info(f"{idle=}")
states.trigger(Transitions.animation_end)

import time
try:
    while True:
        states.loop()
        time.sleep(0.1)
except Exception as e:
    states.trigger(Transitions.stop)
    logger.info(f"exiting. Cause: {e=}")
finally:
    states.trigger(Transitions.stop)



    
