# Civilization Cards

The way purchases of civilization cards work is as follows:

1. The player is presented with all available civilization cards, grouped by type. <- done, but ugly
2. Any cards already owned are marked as such. <- done, as simple text
3. Any cards that have a prerequisite that the player does not own are marked as such. <- needs doing
4. The cost of the cards are marked as original cost / the cost given the credits the player has from other civilization cards. <- done
5. The player selects any number of cards to purchase up to the total value of their  commodity cards on hand and any tokens in treasury (not yet implemented). <- needs doing
6. When the player is ready to purchase, they are then presented with a view to select the combination of commodity cards and treasury tokens to use to purchase the selected cards. <- needs doing
7. The player selects the combination of commodity cards and treasury tokens to use to purchase the selected cards. <- needs doing
8. The cards are purchased and added to the player's civilization cards. <- needs doing
9. The commodity cards are returned to their respective piles. <- needs doing
10. The treasury tokens are returned to the players stock <- needs doing
11. The player is marked as done when it comes to purchasing cards. <- needs doing
12. When all players are done, the game moves to the next activity. <- needs doing
13. Before exiting the activity, the commodity cards are shuffled per the rules and inserted into the piles again. <- needs doing

## Error log
2026-02-21T13:43:47.581741Z  INFO adv_civ::civilization::general_systems: Added human player
2026-02-21T13:43:47.581804Z  INFO adv_civ::civilization::general_systems: Done adding players
2026-02-21T13:43:47.762483Z  INFO adv_civ::civilization::concepts::civ_cards::systems: Another despawn
2026-02-21T13:43:47.762519Z  INFO adv_civ::civilization::concepts::civ_cards::systems: Another despawn
2026-02-21T13:43:47.762530Z  INFO adv_civ::civilization::concepts::civ_cards::systems: Another despawn
2026-02-21T13:43:47.762537Z  INFO adv_civ::civilization::concepts::civ_cards::systems: Another despawn
2026-02-21T13:43:47.762552Z  INFO adv_civ::civilization::concepts::civ_cards::systems: Another despawn
2026-02-21T13:43:47.762560Z  INFO adv_civ::civilization::concepts::civ_cards::systems: Another despawn
2026-02-21T13:43:47.764976Z  WARN bevy_ecs::error::handler: Encountered an error in command `<bevy_ecs::system::commands::entity_command::despawn::{{closure}} as bevy_ecs::error::command_handling::CommandWithEntity<core::result::Result<(), bevy_ecs::world::error::EntityMutableFetchError>>>::with_entity::{{closure}}`: Entity despawned: The entity with ID 995v1 is invalid; its index now has generation 2.
Note that interacting with a despawned entity is the most common cause of this error but there are others
