#/bin/sh

# Enroll players
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/game/request -m "{ \"name\": \"${2}\", \"secret\": \"cockadoodledoo\"}"
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/game/request -m "{ \"name\": \"${3}\", \"secret\": \"cockadoodlegloo\"}"

# Place ships
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${2}/ships/battleship/place -m '{ "coordinates": { "x": 9, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${2}/ships/carrier/place -m '{ "coordinates": { "x": 7, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${2}/ships/destroyer/place -m '{ "coordinates": { "x": 5, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${2}/ships/submarine/place -m '{ "coordinates": { "x": 3, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${2}/ships/patrolboat/place -m '{ "coordinates": { "x": 1, "y": 5 }, "orientation": "Vertical" }'


mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/ships/battleship/place -m '{ "coordinates": { "x": 9, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/ships/carrier/place -m '{ "coordinates": { "x": 7, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/ships/destroyer/place -m '{ "coordinates": { "x": 5, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/ships/submarine/place -m '{ "coordinates": { "x": 3, "y": 5 }, "orientation": "Vertical" }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/ships/patrolboat/place -m '{ "coordinates": { "x": 1, "y": 5 }, "orientation": "Vertical" }'

# Attack - server provides no hand-holding so attacking yourself is perfectly possible.
sleep 0.1

## Destroy battleship
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 9, "y": 5 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 9, "y": 6 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 9, "y": 7 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 9, "y": 8 }'

## Destroy carrier
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 7, "y": 5 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 7, "y": 6 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 7, "y": 7 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 7, "y": 8 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 7, "y": 9 }'

## Destroy destroyer
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 5, "y": 5 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 5, "y": 6 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 5, "y": 7 }'

## Destroy submarine
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 3, "y": 5 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 3, "y": 6 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 3, "y": 7 }'

## Destroy patrolboat
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 1, "y": 5 }'
mosquitto_pub -u "${4}" -h "${1}" -t /${4}/players/${3}/fire -m '{ "x": 1, "y": 6 }'
