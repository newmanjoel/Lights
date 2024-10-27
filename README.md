# Lights
A collection of scripts to controll the lights over my front walkway

## MQTT topics
 general pattern, if you want to request data, use the /request on a topic, all data is ignored. the function will respond on /response.

command/
    lights/
        brightness/
            set/ # floating point number
        color/<id>/set
        location/<id>
    triggers/
        sensor_ 1
        sensor_ 2
        
    system/
        temp
        fps
        stop
        start

    
    

