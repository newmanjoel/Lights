import datetime
import json
import time
import requests
from textual import on
from textual.app import App, ComposeResult
from textual.screen import Screen
from textual.widgets import RichLog
from textual.widgets import Header, Footer

from textual.widgets import RadioSet, RadioButton, ContentSwitcher, Label, Input, Button
from textual.validation import Function, Number, ValidationResult, Validator
from textual.widgets import Markdown

from textual.widgets import Static
from textual.containers import Container
from textual.events import Mount
import logging

from logging_utilities import RichStreamHandler


logger = logging.getLogger("rich")
strm_handler = RichStreamHandler(None, extra=None)
logger.handlers=[]
logger.addHandler(strm_handler)
logger.setLevel(logging.DEBUG)


base_url = "http://192.168.2.24:3000"
debounce_time = 0.5


class Controller(Container):
    local_logger = logger.getChild("Controller")

    def on_mount(self) -> None:
        self.last_input = time.time()
        

    def on_button_pressed(self, event: Button.Pressed) -> None:
        self.local_logger.getChild("button").debug(f"{event}")
        self.send_event()

    def on_input_submitted(self, event: Input.Submitted) -> None:
        self.local_logger.getChild("input").getChild("submitted").debug(f"{event}")
        self.send_event()

    def send_event():
        ...

class AnimationController(Controller):
    DEFAULT_CSS = '#send {width: 80%; align:center middle;} Centered {align: right middle;}'

    def compose(self) -> ComposeResult:
        self.local_logger = logger.getChild("Animation")
        with Container() as container:
            # container.styles.outline = ("outer", "red")
            container.styles.align = ('center','top')
            yield Label("Animation Number", classes='Centered')
            yield Input(
                placeholder="ID of Animation",
                validators=[Number(minimum=3, maximum=5)],
                classes='Centered'
                )
            yield Button("Change Animation", id='send',classes='Centered')
    
    def send_event(self):
        if self.query_one(Input).is_valid and self.last_input + debounce_time < time.time():
            self.last_input = time.time()
            index = self.query_one(Input).value
            response = requests.post(f"{base_url}/animation/{index}")
            if response.status_code != 200:
                self.notify(title="Error Setting Animation", message=f"{response.text}", severity="warning", timeout=10)
                self.local_logger.getChild("send").warning(f"{response.__dict__}")
            
class BrightnessController(Controller):
    DEFAULT_CSS = '#send {width: 100%;}'

    def compose(self) -> ComposeResult:
        self.local_logger = logger.getChild("Brightness")
        with Container():
            yield Label("Brightness Percent")
            yield Input(
                placeholder="Brightness Percent",
                validators=[Number(minimum=0, maximum=100)]
                )
            yield Button("Change Brightness", id='send')
    
    def send_event(self):
        if self.query_one(Input).is_valid and self.last_input + debounce_time < time.time():
            self.last_input = time.time()
            index = self.query_one(Input).value
            response = requests.post(f"{base_url}/animation/brightness/{index}")
            if response.status_code != 200:
                self.notify(title="Error Setting Brightness", message=f"{response.text}", severity="warning", timeout=10)
                self.local_logger.getChild("send").warning(f"{response.__dict__}")
            

class SpeedController(Controller):
    DEFAULT_CSS = '#send {width: 100%;}'

    def compose(self) -> ComposeResult:
        self.local_logger = logger.getChild("Speed")
        with Container():
            yield Label("Speed [FPS]")
            yield Input(
                placeholder="Frames per second",
                validators=[Number(minimum=0, maximum=100)]
                )
            yield Button("Change Speed", id='send')

    def send_event(self):
        if self.query_one(Input).is_valid and self.last_input + debounce_time < time.time():
            self.last_input = time.time()
            index = self.query_one(Input).value
            response = requests.post(f"{base_url}/animation/speed/{index}")
            if response.status_code != 200:
                self.notify(title="Error Settings Speed", message=f"{response.text}", severity="warning", timeout=10)
                self.local_logger.getChild("send").warning(f"{response.__dict__}")


class SwitchableOptions(Container):
    DEFAULT_CSS = 'SwitchableOptions {margin:1 1 1 1;} ContentSwitcher {margin:1; border: round red;}'
    def compose(self) -> ComposeResult:
        yield RadioSet(
            RadioButton(label="Change Animation",value=True, id='change_animation'),
            RadioButton(label="Change Brightness",value=False, id='change_brightness'),
            RadioButton(label="Change Speed",value=False, id='change_speed'),
            id='options'
        )
        with ContentSwitcher(initial='change_animation'):
            yield AnimationController(id='change_animation')
            yield BrightnessController(id='change_brightness')
            yield SpeedController(id='change_speed')
    
    def on_mount(self) -> None:
        self.query_one(RadioSet).border_title = "Options"
        self.query_one(ContentSwitcher).border_title = "Selected Option"
    
    def on_radio_set_changed(self, event: RadioSet.Changed) -> None:
        logger.getChild("radio-changed").debug(f"{event}")
        self.query_one(ContentSwitcher).current = event.pressed.id
        # self.query_one(ContentSwitcher).border_title = event.pressed.value

class Status(Container):
    def compose(self) -> ComposeResult:
        yield Markdown()
    
    def network_update(self) -> None:
        response = requests.get(f"{base_url}/current")
        if response.status_code == 200:
            try:
                data = response.json()
                self.query_one(Markdown).update(f"{data}")
            except Exception as e:
                logger.getChild("Status").getChild("Network").error(f"{e}")
        else:
            logger.getChild("Status").getChild("Network").warning(f"{response.__dict__}")
    def on_mount(self) -> None:
        self.set_interval(
            0.5,
            self.network_update
        )



class LoggingScreen(Screen):
    DEFAULT_CSS = '#logger {dock:bottom; height:70%;} #options {dock:top; width: 100%} #status {dock:top; margin: 1;} #selected_option {dock:bottom;}'
    def compose(self) -> ComposeResult:
        yield Header(id='Header', show_clock=True)
        yield Footer(id='Footer', show_command_palette=True)
        with Container(id='app-grid') as grid:
            grid.styles.layout ='horizontal'
            with Container() as split_left:
                split_left.styles.layout = 'vertical'
                split_left.styles.width ="20%"
                yield SwitchableOptions()
                
            with Container(id='split-center') as split_center:
                split_center.styles.layout = 'vertical'
                yield Status(id='status')
                yield RichLog(markup=True, highlight=True, id='logger')
        # yield Static("Four", classes="box")
    
    def on_mount(self) -> None:
        self.text_log  = self.query_one(RichLog)
        strm_handler.extra = {'callable':self.text_log.write}
        

    
    # @on(Mount)
    # @on(SelectionList.SelectedChanged)
    # def update_selected_view(self) -> None:
    #     self.query_one(Pretty).update(self.query_one(SelectionList).selected)
    



class LayoutApp(App):
    BINDINGS=[("ctrl+v", 'ctrl_v' ,'Toggle Debug Verbosity')]
    

    def on_mount(self) -> None:
        self.push_screen(LoggingScreen())

        self.title = 'Light Controller'
        self.sub_title = 'To control the lights in front of my house'

    def on_callback(self, event):
        #text_log = self.query_one(RichLog)
        #text_log.write(f"{event}")
        return super().on_callback(event)
    
    def action_ctrl_v(self) -> None:
        local_logger = logger.getChild('ctrl+v')
        local_logger.debug('Changing the verbosity')
        level = local_logger.getEffectiveLevel()
        level += 10
        level = level % 60
        local_logger.setLevel(level)
        logger.debug(f'Setting the local logger to {level=}')
        local_logger.debug('debug')
        local_logger.info('info')
        local_logger.warning('warning')
        local_logger.error('error')
        local_logger.critical('critical')

    
    def _on_key(self, event):
        # logger.getChild("on_key").debug(f"{event}")
        # if event.name == 'ctrl_v':
        #     self.on_ctrl_v()
        return super()._on_key(event)
    



if __name__ == "__main__":
    app = LayoutApp()
    app.run()