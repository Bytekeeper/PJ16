use bevy::prelude::*;

use crate::actions::Actions;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cooldown_displays);
    }
}

#[derive(Component)]
pub struct CooldownDisplay(pub Entity);

fn update_cooldown_displays(
    mut display_query: Query<(&mut Text2d, &mut Visibility, &CooldownDisplay)>,
    actions_query: Query<&Actions>,
) {
    for (mut display_text, mut display_visibility, CooldownDisplay(target)) in
        display_query.iter_mut()
    {
        if let Ok(actions) = actions_query.get(*target) {
            if actions.trigger_cooldown.finished() {
                *display_visibility = Visibility::Hidden;
            } else {
                display_text.0 = format!(
                    "{}",
                    actions.trigger_cooldown.remaining_secs().ceil() as u32
                );
                *display_visibility = Visibility::Inherited;
            }
        }
    }
}
