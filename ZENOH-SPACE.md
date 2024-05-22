Set Name

Each unique set of limbo posts is identified by a set name

set-1/

Each set has a laster post with N lasers  and a sensor post with N sensors

Laser N

set-1/laser/1/state=true

Laser N Control State

Control State to turn the laser on / off

set-1/laser/1/state=true

Sensor N

Identify if the sensor is detecting a laser or not

set-1/sensor/1/state=true

For example with a 2 laser/sensor set

set-1/laser/1/state=true
set-1/laser/2/state=true
set-1/sensor/1/state=true
set-1/sensor/2/state=true

when the 1's beam is broken

set-1/sensor/1/state=false

Liveness - for each laser and sensor

{set-name}/sensor|laser/count : u8

{set-name}/sensor|laser/{n}/liveness : liveness token


The app running on the laser post will be configured with a set-name from the ENV

* have a config file which maps the laser numbers to GPIO ports on the device
* bind to 'get' on {set-name}/laser/count which will return the number of lasers in the post
* for each laser

  * subscribe to 'put' on {set-name}/laser/*/state and switch it on/off based on the value
  * bind to 'get' on {set-name}/laser/*/state and will return the current state - true for on and false for off
  * provide a liveness for {set-name}/laser/*/liveness

The app running on the sensor post will be configured with a set-name from the ENV

* have a config file which maps the sensor numbers to GPIO ports on the device
* bind to 'get' on {set-name}/sensor/count which will return the number of sensors in the post
* for each sensor
  * bind to 'get' on {set-name}/sensor/*/state and will return the current state - true for detected siganl and false not
  * provide a liveness for {set-name}/sensor/*/liveness
