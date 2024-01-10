use micro_games_kit::context::GameContext;

pub trait GameObject {
    #[allow(unused_variables)]
    fn activate(&mut self, context: &mut GameContext) {}

    #[allow(unused_variables)]
    fn deactivate(&mut self, context: &mut GameContext) {}

    #[allow(unused_variables)]
    fn update(&mut self, context: &mut GameContext, delta_time: f32) {}

    #[allow(unused_variables)]
    fn draw(&mut self, context: &mut GameContext) {}
}
