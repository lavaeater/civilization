# Cards and calamities

The goal of this round of implementation is to lock down all the rules and behaviors surrounding civilization cards and calamities. Civilization cards affect other parts of the game as well, but for now we will simply focus on their effect on calamities in the game.

## Definitions

### Reducing a city
Removing a city and replacing it with the number of tokens supported by the area that the city occupied.  
### Destroying a city
Removing a city completely - counts as five points in the context of calamity resolution.  
## Calamities to implement

### Second Level

#### Volcano or Earthquake
Already implemented in the game but needs to be amended with the rules for players holding Engineeering:
If the primary victim holds Engineering, an earthquake reduces, rather than destroys, his city. A player who holds Engineering may not
be selected as a secondary victim of an Earthquake. Engineering has no
effect on Volcanoes.