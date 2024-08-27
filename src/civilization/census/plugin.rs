// use crate::GameState;
// use bevy::prelude::{in_state, IntoSystemConfigs, Plugin};
// use crate::civilization::census::components::Census;
// use crate::civilization::census::events::{CensusEnded, CheckAreasForPopulationCommand};
// use crate::civilization::census::systems::{check_areas_for_population, end_census, perform_census};
//
// pub struct CensusPlugin;
//
// impl Plugin for CensusPlugin {
//     fn build(&self, app: &mut bevy::app::App) {
//         app
//             .register_type::<Census>()
//             .add_event::<CensusEnded>()
//             .add_event::<CheckAreasForPopulationCommand>()
//             .add_systems(
//                 bevy::app::Update, (
//                     perform_census
//                         .run_if(in_state(GameState::Playing)),
//                     check_areas_for_population
//                         .run_if(in_state(GameState::Playing)),
//                     end_census
//                         .run_if(in_state(GameState::Playing)),
//                 ),
//             )
//         ;
//     }
// }
//
