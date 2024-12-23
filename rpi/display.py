import logging
import pandas as pd
import numpy as np
import time
import threading
import queue
from functools import partial

import config
from common.file_parser import grb_to_int
from common.common_objects import (
    all_standard_column_names,
    setup_common_logger,
    sanitize_column_names,
)

# used for pushing the data out
# https://github.com/rpi-ws281x/rpi-ws281x-python/blob/master/library/rpi_ws281x/rpi_ws281x.py
# https://github.com/richardghirst/rpi_ws281x/blob/master/ws2811.c
from rpi_ws281x import Color, PixelStrip, ws, RGBW


logger = logging.getLogger("display")
logger = setup_common_logger(logger)
column_names = all_standard_column_names(config.led_num)


def setup() -> PixelStrip:
    # set up the leds
    LED_FREQ_HZ = 800000  # LED signal frequency in hertz (usually 800khz)
    LED_DMA = 10  # DMA channel to use for generating signal (try 10)
    LED_BRIGHTNESS = 255  # Set to 0 for darkest and 255 for brightest
    LED_INVERT = (
        False  # True to invert the signal (when using NPN transistor level shift)
    )
    LED_CHANNEL = 0
    # LED_STRIP = ws.SK6812_STRIP_RGBW
    LED_STRIP = ws.WS2811_STRIP_GRB

    pixels = PixelStrip(
        config.led_num,
        config.led_pin,
        LED_FREQ_HZ,
        LED_DMA,
        LED_INVERT,
        LED_BRIGHTNESS,
        LED_CHANNEL,
        LED_STRIP,
    )
    pixels.begin()
    pixels[0 : config.led_num] = 0
    pixels.show()

    config.frame_rate_arr = np.zeros(1000, dtype=np.float64)

    config.pixels = pixels
    return pixels


def convert_row_to_color(
    input_row: list[int], number_of_columns: int = 1500
) -> list[RGBW]:
    return_list = [0] * (number_of_columns // 3)
    for pixel_num in range(0, number_of_columns, 3):
        led_pixel_index = pixel_num // 3
        led_pixel_color = grb_to_int(
            input_row[pixel_num], input_row[pixel_num + 1], input_row[pixel_num + 2]
        )
        return_list[led_pixel_index] = RGBW(led_pixel_color)
    return return_list


def convert_df_to_list_of_int_speedy(input_df: pd.DataFrame) -> list[list[int]]:
    local_logger = logger.getChild("df_2_int")
    local_logger.debug("starting conversion")
    start_time = time.time()
    working_df = input_df.copy(deep=True)
    time_2 = time.time()
    working_df = sanitize_column_names(working_df)
    working_df.reindex(column_names, axis=1)
    time_3 = time.time()
    raw_data = working_df.to_numpy(dtype=np.ubyte)
    raw_data = raw_data.astype(dtype=np.ubyte)
    time_4 = time.time()

    converter = partial(convert_row_to_color,number_of_columns=750)

    results = np.apply_along_axis(converter, 1, raw_data)
    returned_list = results.tolist()
    end_time = time.time()

    copy_time = time_2 - start_time
    clean_time = time_3 - time_2
    unit_change_time = time_4 - time_3
    enumerate_time = end_time - time_4
    total_time = end_time - start_time

    # Benchmark
    # copy:0.01650 clean:0.04447 types:0.00295 looping:7.64509 total:7.70900
    # after cashing the grb_to_int function
    # copy:0.01680 clean:0.04479 types:0.00313 looping:3.85402 total:3.91874
    # after using numpy apply along axis
    # copy:0.01663 clean:0.04498 types:0.00311 looping:11.00467 total:11.06938
    # doubling down on numpy apply along axis
    # copy:0.01734 clean:0.04529 types:0.00298 looping:10.99190 total:11.05752
    # using np.apply_+along_axis for rows and cashed looping ints
    # copy:0.01702 clean:0.04490 types:0.00296 looping:4.00124 total:4.06612
    # using np.apply_along_axis for frames and looping for rows and casheing all the colors
    # copy:0.01617 clean:0.04324 types:0.00275 looping:2.50638 total:2.56854

    # using the np.apply_along_axis for rows and cashed looping ints as that seems to cleanest/fastest combo

    local_logger.debug(
        f"copy:{copy_time:0.5f} clean:{clean_time:0.5f} types:{unit_change_time:0.5f} looping:{enumerate_time:0.5f} total:{total_time:0.5f}"
    )

    return returned_list


def show_data_on_leds(stop_event: threading.Event, display_queue: queue.Queue) -> None:
    global pixels
    local_logger = logger.getChild("running")
    local_logger.info("Starting")
    data = [100, 0, 0] * config.led_num
    working_df = pd.DataFrame([data], index=range(1), columns=column_names)
    fast_array = convert_df_to_list_of_int_speedy(working_df)
    config.fast_array = fast_array
    led_amount = int(config.led_num)

    while not stop_event.is_set():
        if not display_queue.empty():
            try:
                working_df: pd.DataFrame = display_queue.get()
                config.current_dataframe = working_df
                # working_df = working_df.mul(config.brightness)
                local_logger.info("Changing to new df")
                fast_array = convert_df_to_list_of_int_speedy(working_df)
                # config.fast_array = fast_array
            except queue.Empty as e:
                pass
        # fast_array = config.fast_array
        for row in fast_array:
            if stop_event.is_set() or not display_queue.empty():
                break
            time1 = time.time()
            for led_pixel_index in range(led_amount):
                pixels[led_pixel_index] = row[led_pixel_index]
            time2 = time.time()
            pixels.show()
            time3 = time.time()
            loop_time = time3 - time1
            fps_time = 1.0 / config.fps
            sleep_time = fps_time - loop_time
            if sleep_time < 0:
                sleep_time = 0
            while config.fps == 0:
                time.sleep(0.5)
            else:
                time.sleep(sleep_time)
            time4 = time.time()

            total_time = time4 - time1
            total_fps = 1 / total_time
            config.frame_rate_arr = np.roll(config.frame_rate_arr, 1)
            config.frame_rate_arr[0] = total_fps
            # Loading Array:0.034s Pushing Pixels:0.018s sleeping:0.000s actual_FPS:19.146
            # Loading Array:0.007s Pushing Pixels:0.019s sleeping:0.000s actual_FPS:38.318
            if config.show_fps:
                packing_the_pixels = time2 - time1
                pushing_the_pixels = time3 - time2
                sleeping_time = time4 - time3
                local_logger.debug(
                    f"Loading Array:{packing_the_pixels:.3f}s Pushing Pixels:{pushing_the_pixels:.3f}s sleeping:{sleeping_time:.3f}s actual_FPS:{total_fps:.3f}"
                )
    local_logger.info("Exiting")


pixels = setup()
