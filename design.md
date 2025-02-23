# DESIGN

2 states, setup and running

A resource Grid containing grid size, cell size, etc.

Each cell is an entity that holds:

    - sprite
    - position: vec2,
    - currently_alive: bool,
    - next_alive: bool,
    - neighbours: array<entity; 8>

- Setup: Initialize the grid.
- Update: For each entity look at neighbour values and store
  the next status in future status.
- Update: For each entity look at current value and update the sprite
- Last: For each Future and Current status pair update current.
