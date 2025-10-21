import json
import time
import dash
from dash import Patch
from dash.dependencies import Input, Output
from dash.exceptions import PreventUpdate
from dash import html, callback, dcc
import plotly.express as px
from queue import Queue

from serverobjects import Animation, FrameData, Color, get_animation_at_index, get_locations

import requests



base_url = 'http://192.168.2.24:3000'

animation:Animation = None # type: ignore

animation = get_animation_at_index(base_url, 1) # type: ignore # get a known index so I can display something at the start
locations = get_locations(base_url)

# location has [id, x, y]

fig = px.scatter(locations, x="x", y="y")
# fig.update_xaxes(title_text="Sepal Width (cm)", range=[-650, 50])
# fig.update_yaxes(title_text="Sepal Length (cm)", range=[-650, 50])


app = dash.Dash(__name__)
app.layout = html.Div([
    dcc.Store(id='session-data-store', storage_type='session'),
    html.P("This is some text"),
    html.Div(id="updateable-div"),
    dcc.Graph(id="scatter-plot", figure=fig, style={'height': '500px', 'width': '500px'} ),
    html.Div(children=[
        html.P("Brightness"),
        dcc.Slider(min=0, max=100, step=1, id="Brightness-Slider"),
        html.Button("Change Brightness", id="change-brightness-button")
        
    ]),
    dcc.Interval(
        id='interval-component',
        interval= 33,  # 30 fps
        n_intervals=0
    ),

])

@app.callback(
    Output("scatter-plot", "figure", allow_duplicate=True),
    Input('session-data-store', 'data'),
    prevent_initial_call=True
)
def patch_colors(session_data):
    # start_time = time.time()
    animation_index = session_data.get("animation_index", 1)
    global animation
    if (animation is None) or (animation.id != animation_index):
        animation = get_animation_at_index(base_url, animation_index) # type: ignore
        if animation is None:
            raise PreventUpdate(f"Bad Animation Index of {animation_index}")
        
    frame = animation.frames[session_data.get("frame_index",0)]
    patch = Patch()
    patch["data"][0]["marker"]["color"] = frame.hex_colors
    # end_time = time.time()
    # print(f"{end_time-start_time:0.6f} seconds to render")
    return patch

@callback([
    Output(component_id="session-data-store", component_property="data"),
    Output(component_id="updateable-div", component_property="children")],
    Input(component_id="interval-component", component_property="n_intervals"))
def update_from_server(n_intervals:int):
    current = requests.get(f"{base_url}/current")
    # print(f"{current.text=}")
    if current.status_code == 200:
        current_data = current.json()
        j_data = json.dumps(current_data)
        
        return (current_data, j_data)
    raise PreventUpdate 


app.run(debug=True)