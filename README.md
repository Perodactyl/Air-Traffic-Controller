# ATC

Air Traffic Controller (ATC) is a game about airspace micromanagement. This version is a mostly-the-same remake in Rust of the [bsdgames version](https://github.com/vattam/BSDGames).

## Compiling from source
```sh
git clone https://github.com/Perodactyl/Air-Traffic-Controller
cargo build
```

## Gameplay
When you first launch ATC, you will see the radar view and status panel. Beneath the game grid is also the command input, but it starts empty and, as such, invisible.
### Radar View
The radar view shows an overhead map of your airspace, with north pointed up. Several symbols denote objects within:
- Blank Space: Indicated by `.`. Does nothing.
- Path Marker: Indicated by `+`. Does nothing, but serves as a visual aid.
- Exit: Indicated with a number on the edge. Planes can enter your airspace here at 7000ft and can exit it at 9000ft.
- Airport: Indicated with a directional caret (`^`, `<`, `>`, `v`) followed by an ID number. Some planes must be directed to take off from here, while others must land here. To launch a plane, set its altitude to any value above 0ft. The directional caret denotes the runway's direction. Planes taking off will launc beyond the runway and planes landing must come in from the back of the runway.
- Beacon: Indicated by `*` followed by an ID number. These can be used to specify when a plane should perform an action.
- Airplane: Indicated with a letter followed by a flight level number. The number represents the plane's flight level in thousands of feet. If the letter is lowercase, the plane is a jet, but if it is uppercase, the plane is a prop plane which moves at half speed.

### Status Panel
The status panel provides more information about each plane. At the top, it shows the current time in cycles and your score (number of planes safely landed or directed to an exit). Afterward, a listing of each plane is shown. The first column shows the plane's name (and its location if it is landed), the second shows where you must send it, and the third shows a queued command.

### Command Input
To direct plane, first enter its callsign letter. Capitalization does not matter. Then, enter an action.
- [x] Altitude (`A`): Sets or changes the plane's target flight level. Planes can only move one flight level up or down each time per movement tick. Next arguments:
    - [x] Digit: Send plane to this flight level.
    - [x] `-` (or `_`) digit: Send the plane down by this many flight levels.
    - [x] `+` (or `=`) digit: Send the plane up by this many flight levels.
- [x] Heading (`H` or `T`): Sets the plane's direction. Planes can only turn 90 degrees each time they move. If the turn is greater than 90 degrees, the plane will turn 90 degrees on the first movement tick and 45 degrees on the next, leading to an overshoot. For 180 degree turns, the plane will always turn clockwise. Next arguments:
    - [x] Direction: can be input with the keys surrounding S (`Q`, `W`, `E`, `A`, `D`, `Z`, `X`, `C`), the numpad keys surrounding 5, or vim bindings. When using vim bindings, `I`, `J`, `K`, and `L` are used for cardinal directions and the key above `U`, `I`, `O`, or `P` is the ordinal direction 45 degrees clockwise.
    - [ ] `T`: Turn **T**oward an object on the radar. Not yet implemented.
- [x] Circle (`C`): Causes the plane to move in a 4-space pattern until otherwise commanded with a new heading. Use this if you do not have time to handle a plane or if it needs to finish changing altitudes before continuing. Next arguments:
    - [x] Clockwise (`Q` or `[`) & Counter-Clockwise (`E` or `]`): Specifies the direction the plane will circle in. If unspecified, the default is clockwise.
    - [ ] Digit: Sets the number of cycles before the plane will continue. Not yet implemented.
- [x] Set visibility (`U`, `M`, and `I`): Changes the visibility of the current plane:
    - [x] Unmark (`U`): Dims the plane from view until it reaches a site where it has a delayed action. Use this if a plane has an instruction, but will later need more before it can reach its destination.
    - [x] Mark (`M`): Undoes an Unmark or Ignore command.
    - [x] Ignore (`I`): Dims the plane from view. Use this if a plane will safely reach its destination on its own.
After specifying a command, you can optionally specify a <u>delay</u>:
    - [x] At (`A`) digit: Command will run when the plane arrives at the beacon with a matching ID number.
    - [ ] In (`I`) digit: Command will run after the plane moves *digit* times. Not yet implemented.
You can also specify additional commands with `;` or `&`. Commands after a delayed command will be held up until the first command in the chain finishes.

### Example Commands
| Keystrokes | Action |
| :--------- | -----: |
| `aa9`      | Send plane A to flight level 9 (9000ft). |
| `aa=2`, `aa+2` | Send plane A 2 flight levels higher. |
| `atw`      | Turn plane A so it moves due north. |
| `atwa1`, `atw@1` | Turn plane A to the north when it arrives at beacon _*1_. |
| `au`       | Unmark plane A until it arrives at the beacon. |
| `ataa1;atxa0` | Turn plane A west when it arrives at beacon _*1_, then turn it south once it arrives at beacon _*0_. |


