import datetime
from time import sleep, time
from rich import print
# from rich.console import Group
from rich.panel import Panel
from rich.live import Live
from rich.align import Align
from rich import box
from rich.console import Console, Group
from rich.tree import Tree

from rich.layout import Layout
import logging

from rich.logging import RichHandler

FORMAT = "%(message)s"
log = logging.getLogger("rich")

selected_option_index = 0


def create_logging_panel() -> Panel:
    message = f'The time is {datetime.datetime.now()}'
    message_panel = Panel(
        Align.center(
            Group("\n", Align.center(message)),
            vertical="middle",
        ),
        box=box.ROUNDED,
        padding=(1, 2),
        title="[b red]Thanks for trying out Rich!",
        border_style="bright_blue",
    )
    return message_panel

def assign_logging_panel(layout:Layout):
    layout.update(create_logging_panel())

def create_options_panel() -> Panel:
    tree_options = [
        "Change Brightness",
        "Change Animation",
        "Change Speed"
    ]
    tree = Tree("Options")
    for index,text in enumerate(tree_options):
        working_node = tree.add(text)
        if index == selected_option_index:
            working_node.label = f"[bold red]* {working_node.label} (Selected)"

    return Panel(tree)

def assign_options_panel(layout:Layout):
    layout.update(create_options_panel())

def create_layout() -> Layout:
    layout = Layout()
    layout.split_row(
        Layout(name='options'),
        Layout(name='right', ratio=2)
    )
    layout['right'].split_column(
        Layout(name='status'),
        Layout(name='logging')
    )
    assign_logging_panel(layout['logging'])
    return layout

def update_layout(layout: Layout) -> Layout:
    assign_logging_panel(layout['logging'])
    assign_options_panel(layout['options'])
    return layout

def main():
    log.info("Hello, World!", extra={"markup":True})
    layout = create_layout()
    fps = 1
    with Live(layout , refresh_per_second=fps) as live:
        while True:
            try:
                live.update(update_layout(layout))
            except KeyboardInterrupt:
                break

if __name__ == "__main__":
    logging.basicConfig(
        level="NOTSET", format=FORMAT, datefmt="[%X]", handlers=[RichHandler()]
    )

    main()