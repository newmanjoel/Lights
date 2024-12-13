
__all__ = ['Light Display']

from dataclasses import dataclass, field
import heapq
import pygame
import pygame_menu
import pygame_menu.utils as ut
import random
import time

from collections import deque
from math import inf
from typing import List, Union, Optional, Tuple, Any, Generator


# Define some colors
BACKGROUND = (34, 40, 44)
BLACK = (0, 0, 0)
BLUE = (0, 0, 255)
BROWN = (186, 127, 50)
DARK_BLUE = (0, 0, 128)
DARK_GREEN = (0, 128, 0)
GREEN = (0, 255, 0)
GREY = (143, 143, 143)
LIGHT_BLUE = (0, 111, 255)
RED = (255, 0, 0)
WHITE = (255, 255, 255)


@dataclass
class Point3():
    x: float
    y: float
    z: float = field(default=0)


@dataclass
class Point2():
    x: float
    y: float


@dataclass
class Color():
    r: int
    g: int
    b: int

# noinspection PyDefaultArgument


@dataclass
class Light(object):
    location: Point3
    color: Color


class LightViz(object):
    _height: int
    _margin: int
    _menu: 'pygame_menu.Menu'
    _mouse_drag: bool
    _offset: Point2
    _rows: int
    _visualize: bool
    _width: int
    _screen: pygame.surface

    def __init__(self) -> None:

        # Used for handling click & drag
        self._mouse_drag = False


        # Create the window
        self._clock = pygame.time.Clock()
        self._fps = 60
        self._surface = pygame.display.set_mode((900, 650))
        pygame.display.set_caption("3D Animation Viewer")
        # self._surface = create_example_window('Example - Maze', (900, 650))
        

        # Setups the menu
        self._setup_menu()

    def _setup_menu(self) -> None:
        """
        Setups the menu.
        """

        # Creates the events
        # noinspection PyUnusedLocal
        def onchange_dropselect(*args) -> None:
            """
            Called if the select is changed.
            """
            b = self._menu.get_widget('run_generator')
            b.readonly = False
            b.is_selectable = True
            b.set_cursor(pygame_menu.locals.CURSOR_HAND)

        def button_onmouseover(w: 'pygame_menu.widgets.Widget', _) -> None:
            """
            Set the background color of buttons if entered.
            """
            if w.readonly:
                return
            w.set_background_color((98, 103, 106))

        def button_onmouseleave(w: 'pygame_menu.widgets.Widget', _) -> None:
            """
            Set the background color of buttons if leaved.
            """
            w.set_background_color((75, 79, 81))

        def button_onmouseover_clear(w: 'pygame_menu.widgets.Widget', _) -> None:
            """
            Set the background color of buttons if entered.
            """
            if w.readonly:
                return
            w.set_background_color((139, 0, 0))

        def button_onmouseleave_clear(w: 'pygame_menu.widgets.Widget', _) -> None:
            """
            Set the background color of buttons if leaved.
            """
            w.set_background_color((205, 92, 92))

        def _visualize(value: bool) -> None:
            """
            Changes visualize.
            """
            self._visualize = value

        theme = pygame_menu.Theme(
            background_color=pygame_menu.themes.TRANSPARENT_COLOR,
            title=False,
            widget_font=pygame_menu.font.FONT_FIRACODE,
            widget_font_color=(255, 255, 255),
            widget_margin=(0, 15),
            widget_selection_effect=pygame_menu.widgets.NoneSelection()
        )
        self._menu = pygame_menu.Menu(
            height=100,#self._screen_width,
            mouse_motion_selection=True,
            position=(645, 25, False),
            theme=theme,
            title='Menu Title',
            width=240
        )

        self._menu.add.toggle_switch(
            'Visualize',
            self._visualize,
            font_size=20,
            margin=(0, 5),
            onchange=_visualize,
            state_text_font_color=((0, 0, 0), (0, 0, 0)),
            state_text_font_size=15,
            switch_margin=(15, 0),
            width=80
        )

        self._menu.add.label(
            'menu label',
            font_name=pygame_menu.font.FONT_FIRACODE_BOLD,
            font_size=22,
            margin=(0, 5)
        ).translate(-12, 0)
        self._menu.add.dropselect(
            title='menu drop_select',
            items=[('Prim', 0),
                   ('Alt Prim', 1),
                   ('Recursive', 2),
                   ('(+) Terrain', 3)],
            dropselect_id='generator',
            font_size=16,
            onchange=onchange_dropselect,
            padding=0,
            placeholder='Select one',
            selection_box_height=5,
            selection_box_inflate=(0, 20),
            selection_box_margin=0,
            selection_box_text_margin=10,
            selection_box_width=200,
            selection_option_font_size=20,
            shadow_width=20
        )
        self._menu.add.vertical_margin(10)
        btn = self._menu.add.button(
            'menu button',
            self._run_generator,
            button_id='run_generator',
            font_size=20,
            margin=(0, 30),
            shadow_width=10,
        )
        btn.readonly = True
        btn.is_selectable = False

        # Create about menu
        menu_about = pygame_menu.Menu(
            height=self._screen_width + 20,
            mouse_motion_selection=True,
            position=(645, 8, False),
            theme=theme,
            title='menu about',
            width=240
        )
        menu_about.add.label('pygame-menu\nMaze', font_name=pygame_menu.font.FONT_FIRACODE_BOLD, font_size=25,
                             margin=(0, 5))
        menu_about.add.vertical_margin(10)
        text = 'Left click to create a wall or move the start and end points.\n' \
               'Hold left CTRL and left click to create a sticky mud patch (whi' \
               'ch reduces movement speed to 1/3).\n'
        text += 'The point of these mud patches is to showcase Dijkstra\'s algor' \
                'ithm (first) and A* (second) by adjusting the "distances" betwe' \
                'en the nodes.\n\n'
        text += 'After a pathfinding algorithm has been run you can drag the sta' \
                'rt/end points around and see the visualisation update instantly' \
                ' for the new path using the algorithm that was last run.\n'
        menu_about.add.label(text, font_name=pygame_menu.font.FONT_FIRACODE, font_size=12,
                             margin=(0, 5), max_char=-1, padding=0)
        menu_about.add.label('License: GNU GPL v3.0', margin=(0, 5),
                             font_name=pygame_menu.font.FONT_FIRACODE, font_size=12)
        menu_about.add.url('https://github.com/ChrisKneller/pygame-pathfinder', 'ChrisKneller/pygame-pathfinder',
                           font_name=pygame_menu.font.FONT_FIRACODE, font_size=12,
                           font_color='#00bfff')
        menu_about.add.vertical_margin(20)
        menu_about.add.button(
            'Back',
            pygame_menu.events.BACK,
            button_id='about_back',
            cursor=pygame_menu.locals.CURSOR_HAND,
            font_size=20,
            shadow_width=10
        )

        btn = self._menu.add.button(
            'About',
            menu_about,
            button_id='about',
            float=True,
            font_size=20,
            margin=(0, 75),
            shadow_width=10
        )
        btn.translate(50, 0)

        # Configure buttons
        for btn in self._menu.get_widgets(['run_generator', 'run_solver', 'about', 'about_back']):
            btn.set_onmouseover(button_onmouseover)
            btn.set_onmouseleave(button_onmouseleave)
            if not btn.readonly:
                btn.set_cursor(pygame_menu.locals.CURSOR_HAND)
            btn.set_background_color((75, 79, 81))

    def _run_generator(self) -> None:
        """
        Run the generator.
        """
        if self._visualize:
            ut.set_pygame_cursor(pygame_menu.locals.CURSOR_NO)
        o_visualize = self._visualize
        gen_type = self._menu.get_widget('generator').get_value()[1]
        if gen_type != 3:
            self._clear_maze()
        if gen_type == 0:
            self._grid = self._prim(start_point=self._start_point)
        elif gen_type == 1:
            self._grid = self._better_prim(start_point=self._start_point)
        elif gen_type == 2:
            pygame.display.flip()
            self._recursive_division()
        elif gen_type == 3:
            self._random_terrain()
        self._visualize = o_visualize
        if self._visualize:
            ut.set_pygame_cursor(pygame_menu.locals.CURSOR_ARROW)

    @staticmethod
    def _sleep(ms: float) -> None:
        """
        Sleep time.

        :param ms: Sleep time in milliseconds
        """
        time.sleep(ms)

    def _update_gui(self, draw_background=True, draw_menu=True, draw_grid=True) -> None:
        """
        Updates the gui.

        :param draw_background: Draw the background
        :param draw_menu: Draw the menu
        :param draw_grid: Draw the grid
        """
        if draw_background:
            # Draw a black background to set everything on
            self._surface.fill(BACKGROUND)

        if draw_grid:
            # Draw the grid
            for row in range(self._rows):
                for column in range(self._rows):
                    self._draw_square(self._grid, row, column)

        if draw_menu:
            self._menu.draw(self._surface)

    @staticmethod
    def _quit() -> None:
        """
        Quit app.
        """
        global running
        running = False
        pygame.quit()
        exit()

    def draw_frame(self) -> None:
        self._surface.fill((0, 0, 0))  # Clear the screen with black
        if 0 <= frame_id < len(animation.frames):
            colors = animation.frames[frame_id]
            # print(f"{colors=}, {frame_id=}")
            for point, color in zip(locations, colors):
                # Assuming point is in the format [x, y, z]
                x, y, z = (point.x, point.y, point.z)
                r, g, b = color
                # Project z-axis by scaling the size of the circle
                # Adjust the divisor for different depth scaling
                size = max(1, 5 - (z / 50))

                #                (surface, (color)        , center,           , radius, width)
                pygame.draw.circle(screen, (r, g, b),
                                (x + 400, y + 300), int(size))

    def handle_mouse_down(self, event: pygame.event) -> None:
        print(f"Mouse Down @ {event.get("pos")=}")
        self._mouse_drag = True

    def handle_mouse_drag(self, event: pygame.event) -> None:
        if self._mouse_drag:
            print(f"Mouse Drag @ {event.get("pos")=}")

    def handle_mouse_up(self, event: pygame.event) -> None:
        print(f"Mouse Down @ {event.get("pos")=}")
        self._mouse_drag = False

    def run(self) -> None:
        global running
        while running:
            events = pygame.event.get()

            # Update the menu
            self._menu.update(events)
            for event in events:

                if event.type == pygame.QUIT:
                    self._quit()

                if event.type == pygame.MOUSEBUTTONDOWN:
                    self.handle_mouse_down(event)
                elif event.type == pygame.MOUSEMOTION:
                    self.handle_mouse_drag(event)
                elif event.type == pygame.MOUSEBUTTONUP:
                    self.handle_mouse_up(event)

        # Update the app
        self._update_gui()
        draw_frame(current_frame_id)
        clock.tick(animation.speed)
        # Update clock
        self._clock.tick(self._fps)

    pygame.quit()


def main(test: bool = False):
    """
    Main function.

    :param test: Indicate function is being tested
    :return: App
    """
    global running
    running = False
    app = LightViz()
    app.run()
    return app


if __name__ == '__main__':
    main()
