# States
night_idle -- to_house --> wave_on
night_idle -- to_street --> wave_on
wave_on -- animation_end --> steady_on

steady_on -- stop --> idle
steady_on -- timeout --> wave_off
wave_off -- animation_end --> idle

idle -- time_of_day --> day_idle
idle -- time_of_day --> night_idle
day_idle -- time_of_day --> idle
night_idle -- time_of_day --> idle

## state descriptions
### idle
intermediate state to decide between day and night cycles.

### night_idle
low light, almost no animation.

### day_idle
all lights are off.

### wave_on
animation that turns on the light (to the same colour as the steady_on). Starting from the direction of the trigger point going away. 

This might be a randomized animation. TBD.

### wave_off
animation that turns off the lights (to the same colour as idle)

### steady_on
lights are just on to a fixed colour. No animation while its on. colour can change however but only steady_on entry






