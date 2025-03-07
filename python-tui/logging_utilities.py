import logging
import colorlog


def setup_root_logger():
    color_formatter = colorlog.ColoredFormatter(
        "%(log_color)s%(asctime)s %(name)s - %(levelname)s - %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        log_colors={
            "DEBUG": "cyan",
            "INFO": "green",
            "WARNING": "yellow",
            "ERROR": "red",
            "CRITICAL": "bold_red",
        },
    )
    logger = logging.getLogger()
    console_handler = logging.StreamHandler()
    console_handler.setFormatter(color_formatter)
    logger.addHandler(console_handler)
    logger.setLevel(logging.DEBUG)

class RichStreamHandler(logging.StreamHandler):
    def __init__(self, stream=None, extra=None):
        super().__init__(stream)
        self.formatter = logging.Formatter('%(levelname)s*%(name)s - %(levelname)s - %(message)s')
        self.setFormatter(self.formatter)
        if extra is not None:
            self.extra = extra
        else:
            self.extra = {}

    def emit(self, record):
        msg = self.format(record)
        # Custom behavior here, e.g., add prefixes, send to different streams based on level

        msg = msg.replace("ERROR*", "[red]")
        msg = msg.replace("WARNING*", "[yellow]")
        msg = msg.replace("INFO*", "[green]")
        msg = msg.replace("DEBUG*", "[cyan]")
        msg = msg.replace("CRITICAL*", "[bold red]")

        func = self.extra.get('callable',None)
        if func is not None:
            func(msg)
        self.flush()

# setup_root_logger()