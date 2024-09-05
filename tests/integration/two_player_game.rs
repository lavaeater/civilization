mod common;

use bevy::prelude::{Events, NextState, Update};
use bevy::prelude::NextState::Pending;
use bevy_game::civilization::city_support::plugin::{check_city_support, CheckPlayerCitySupport};
use bevy_game::civilization::game_phases::game_activity::GameActivity;
use bevy_game::civilization::general::components::BuiltCity;
use common::{setup_player, setup_bevy_app};
use crate::common::create_area;

/***
We are going to write a test that actually plays the game for us with two players. It is going
to be sooo much work, but perhaps it will be worth it?

It is either this or making some kind of scripting for the commands. I will do these in parallell...
 */