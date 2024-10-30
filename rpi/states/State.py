from dataclasses import dataclass, field
from datetime import datetime
from typing import Callable
from Transitions import Transitions

import logging

# Add the root directory to the Python path
import os, sys

current_directory = os.path.dirname(os.path.abspath(__file__))
webservers_directory = os.path.abspath(os.path.join(current_directory, ".."))
sys.path.append(webservers_directory)

from common.common_objects import (
    setup_common_logger,
)

logger = logging.getLogger("state")
logger = setup_common_logger(logger)


def defualt_on_entry(state, what_happened:Transitions = Transitions.unknown) -> None:
    logger.getChild(state.name).debug(f"entering {state.name} due to {what_happened}")


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
        # logger.getChild('check_triggers').debug(f'{self.triggering=}')
        for func in self.triggering:
            # logger.getChild('check_triggers').debug(f"checking {func=}")
            result = func(self)
            if result is not None:
                return result
        return None
