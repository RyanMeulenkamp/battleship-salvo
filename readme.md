# Battleship: Salvo

## Rules of the game

The rules of standard battleship are as follows:

* Each player starts off with the following ships:
  * Carrier - 5 units
  * Battleship - 4 units
  * Destroyer - 3 units
  * Submarine - 3 units
  * Patrol boat - 2 units
* Both players place their ships around a 10 x 10 map.
  * This map has coordinates: column's 1 - 10 and rows A - J (game uses coordinates transposed to those of excel sheets).
  * Ships can be placed next to each other, but overlapping is not allowed.
* Each turn, a player announces a target coordinate, and the target player responds with hit or miss
* If all squares of the target player are hit, the ship sinks. The target player announces which ship has sunk.
* The game ends when a player has lost all of his ships.

The salvo edition of battleship has an interesting twist to these rules:

* A player can, instead of choosing one coordinate, choose multiple shots, a salvo.
* The number of shots he is allowed to take is the same as the number of ships he has left. So that is 5 from the start.
* The target only announces how many of the shots were hits/misses. It takes a bit more thinking to figure out which ones were hits.

We have some modifications of our own to this ruleset:

* Instead of only choosing x and y coordinates, you also choose which player to attack, for each individual shot of the salvo.
* The player that is under attack can only see the coordinates on his map. Not the coordinates of the other players under attack. The total number of hits and sinking of ships is open to anyone though.
* Subsequently, the game ends when one player is left.
* To keep the game at least somewhat fair, we have a single source of truth that announces hits, misses and sinks. That is the server application.

## Technical/protocol/state machine description

We will supply a server application that hooks into a MQTT broker. Each team will implements it's own client. Off course, changes to the server application are allowed, in consultation with other teams and us. We will be the maintainers of the server side application. After the game, each team can show the inner workings of its client application. Messages with more than one value be formatted as JSON. We will provide a protobuf file for this.

Description

* Game state is published and retained at topic `/game/state`
* `lobby` state
  * Each player gets the chance to start its application and request participation by publishing its teamname plus a secret on `/game/request_participation`
    * This is private topic. Only the server can subscribe to it. That way, only the server will know the secret for every team.
    * Message format: `{"name": "<teamname>", "secret", "<some_secret>"}`, example `{"name": "Henkiebunch", "secret": "cockadoodledoo"}`.
    * Private messages will be encrypted. From now on we will mention when a message is expected to be encrypted.
      * Algorithm is `argon2id13`
      * Encoded into `base64` string
    * Open messages that have to be guaranteed to have come from the right player will be signed with an encrypted copy of the message body This is placed inside another JSON message.
      * Example: `{"payload": {"a": "b", "c": "d"}, "signature": "LRsNengquZ27OARrdN6ZuDjMIlitEopytYBYNeEcYscvNw3rld+fnr+iC0os/n8AhzhUoGYdNkkKAxqLPfzTRw+wI5wO6+U7m+t6fA=="}`
  * Number of players is published at `/players/count`.
  * A list of player names is published at `/players/list`, in the form of JSON array `[ "<team_one>", "<team_two>" ]`
  * Each player provides coordinates for their ships on `/players/<player_name>/ships/<ship>/place`. Message format: `{"coordinates": "<x>;<y>", "Orientation": "<HORIZONTAL|VERTICAL>"}`.
    * These messages are encrypted.
    * The coordinates of the ship are counted from the same corner as your game board starts. So, say your game board starts counting in the top-left corner, so does the ship.
  * Feedback in the form of a boolean can be found at `/players/<player_name>/ships/<ship>/approved`
  * If a problem occurred with the latest ship placement, an error message will be published on `/players/<player_name>/ships/<ship>/error`
  * From the moment the latest player was added, you get one minute to generate a map and provide the server with all the ships on your map.
  * Play order is counting up from first correct map provider to the last. The starting player is determined randomly though.
* `turn` state
  * The server chooses the first player at random and game state goes to `turn` immeditately
  * Current player is published every turn at `/game/current` and retained until the next turn.
  * Player provides a number of shots to fire on topic `/players/<target_player_name>/fire`, in the form of `{ "data": {"x": <x>, "y": <y>}, "signature": "<signature>"}`
  * After each shot, the target player receives the coordinates on `/players/<target_player_name>/hit`. Format `{ "x": <x>, "y": <y> }`.
  * After the maximum shots of the current player is received, the server will publish the total number of actual hits at `/game/hits`.
  * If a ship sinks, boolean `true` is published on `/player/<player_name>/ships/<ship>/sunk`
  * If a player was defeated, boolean `true` is published at `/player/<player_name>/defeated`
  * If all but one player were defeated, the game is over. `/game/state` goes to `over`.
* `over` state
  * Winner is published at `/game/winner`.
  * Stats can be found under `/game/stats/*`
  * Game state is reset to `lobby` after 5 minutes.

For ther record: the server only maintains the right amount of information to enforce a fair game. It does not hold your hand. It keeps no state of what places you already shot. It even allows you to shoot the same place on a ship twice.

