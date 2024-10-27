from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Callable
from Transitions import Transitions

import logging

logger = logging.getLogger("state")


def defualt_on_entry(state, what_happened:Transitions = Transitions.unknown) -> None:
    global state_timer
    logger.getChild(state.name).debug(f"entering {state.name} due to {what_happened}")
    state_timer = datetime.now() + timedelta(minutes=1)


def default_on_exit(state, what_happened:Transitions = Transitions.unknown) -> None:
    logger.getChild(state.name).debug(f"leaving {state.name} due to {what_happened}")


@dataclass
class State():
    name:str
    on_exit_func: Callable = default_on_exit
    on_entry_func: Callable = defualt_on_entry
    triggering: list[Callable] = field(default_factory=list) 

    def on_entry(self, what_happened:Transitions = Transitions.unknown) -> None:
        self.on_entry_func(self,what_happened)

    def on_exit(self, what_happened:Transitions = Transitions.unknown) -> None:
        self.on_exit_func(self, what_happened)
    
    def check_triggers(self) -> Transitions | None:
        for func in self.triggering:
            result = func(self)
            if result is not None:
                return result
        return None