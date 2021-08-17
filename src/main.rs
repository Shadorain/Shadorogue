/* ------------------------------- Shadoblade ------------------------------- */
// -- Includes -- {{{
extern crate serde;
use rltk::{GameState, Rltk, Point};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};

/* Resources */
mod components;
pub use components::*;
mod player;
pub use player::*;
mod rect;
pub use rect::Rect;
mod gui;
mod gamelog;
mod spawner;
mod random_table;
mod rex_assets;
pub mod camera;
/* Systems */
mod visibility_sys;
pub use visibility_sys::VisibilitySystem;
mod map_indexing_sys;
pub use map_indexing_sys::MapIndexingSystem;
mod melee_combat_sys;
use melee_combat_sys::MeleeCombatSystem;
mod dmg_sys;
use dmg_sys::DamageSystem;
mod inventory_sys;
use inventory_sys::*;
mod saveload_sys;
mod particle_sys;
use particle_sys::*;
mod hunger_sys;
mod trigger_sys;
pub mod gamesys;
pub use gamesys::*;
mod lighting_sys;
use lighting_sys::*;
mod movement_sys;
use movement_sys::*;
/* Modules */
pub mod map_builders;
pub mod raws;
pub mod map;
use map::*;

mod ai;
mod spatial;

#[macro_use]
extern crate lazy_static;
/* }}} */
// -- State -- {{{
const SHOW_MAPGEN_VISUALIZER : bool = false;

#[derive(PartialEq, Copy, Clone)]
pub enum VendorMode { Buy, Sell }

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    Ticking,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range:i32, item : Entity },
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame,
    NextLevel,
    PreviousLevel,
    TownPortal,
    ShowRemoveEquipment,
    GameOver,
    MagicMapReveal { row : i32 },
    MapGeneration,
    ShowCheatMenu,
    ShowVendor { vendor: Entity, mode: VendorMode },
    TeleportingToOtherLevel { x:i32, y:i32, depth:i32 },
}

pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn run_systems (&mut self) {
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut encumbrance = ai::EncumbranceSystem{};
        encumbrance.run_now(&self.ecs);
        let mut initiative = ai::InitiativeSystem{};
        initiative.run_now(&self.ecs);
        let mut turnstatus = ai::TurnStatusSystem{};
        turnstatus.run_now(&self.ecs);
        let mut quipper = ai::QuipSystem{};
        quipper.run_now(&self.ecs);
        let mut adjacent = ai::AdjacentAI{};
        adjacent.run_now(&self.ecs);
        let mut visible = ai::VisibleAI{};
        visible.run_now(&self.ecs);
        let mut approach = ai::ApproachAI{};
        approach.run_now(&self.ecs);
        let mut flee = ai::FleeAI{};
        flee.run_now(&self.ecs);
        let mut chase = ai::ChaseAI{};
        chase.run_now(&self.ecs);
        let mut defaultmove = ai::DefaultMoveAI{};
        defaultmove.run_now(&self.ecs);
        let mut moving = MovementSystem{};
        moving.run_now(&self.ecs);
        let mut triggers = trigger_sys::TriggerSystem{};
        triggers.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut itemuse = ItemUseSystem{};
        itemuse.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);
        let mut equip_remove = EquipmentRemoveSystem{};
        equip_remove.run_now(&self.ecs);
        let mut hunger = hunger_sys::HungerSystem{};
        hunger.run_now(&self.ecs);
        let mut particles = particle_sys::ParticleSpawnSystem{};
        particles.run_now(&self.ecs);
        let mut lighting = LightingSystem{};
        lighting.run_now(&self.ecs);

        self.ecs.maintain();
    }
}
impl GameState for State {
    fn tick (&mut self, ctx : &mut Rltk) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }
        ctx.cls();
        particle_sys::cull_dead_particles(&mut self.ecs, ctx);

        match newrunstate {
            RunState::MainMenu {..} => {},
            RunState::GameOver {..} => {},
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            },
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            } RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            } RunState::Ticking => {
                while newrunstate == RunState::Ticking {
                    self.run_systems();
                    self.ecs.maintain();
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => newrunstate = RunState::AwaitingInput,
                        RunState::MagicMapReveal { .. } => newrunstate = RunState::MagicMapReveal { row: 0 },
                        RunState::TownPortal => newrunstate = RunState::TownPortal,
                        RunState::TeleportingToOtherLevel { x, y, depth } => newrunstate = RunState::TeleportingToOtherLevel { x, y, depth },
                        _ => newrunstate = RunState::Ticking,
                    }
                }
            } RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting { range: is_item_ranged.range, item: item_entity };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item: item_entity, target: None })
                                .expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        }
                    },
                }
            } RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item: item_entity })
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    },
                }
            } RunState::ShowTargeting{range,item} => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: result.1 })
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    },
                }
            } RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => newrunstate =
                        RunState::MainMenu { menu_selection: selected },
                    gui::MainMenuResult::Selected { selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            gui::MainMenuSelection::LoadGame => {
                                saveload_sys::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                saveload_sys::delete_save();
                            },
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); },
                        }
                    },
                }
            } RunState::ShowCheatMenu { .. } => {
                let result = gui::show_cheat_menu(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => { },
                    gui::CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    },
                    gui::CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        newrunstate = RunState::AwaitingInput;
                    },
                    gui::CheatMenuResult::Reveal => {
                        let mut map = self.ecs.fetch_mut::<Map>();
                        for v in map.revealed_tiles.iter_mut() {
                            *v = true;
                        };
                        newrunstate = RunState::AwaitingInput;
                    },
                    gui::CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut player_pools = pools.get_mut(*player).unwrap();
                        player_pools.god_mode = true;
                        newrunstate = RunState::AwaitingInput;
                    },
                }
            } RunState::ShowVendor { vendor, mode } => {
                let result = gui::show_vendor_menu(self, ctx, vendor, mode);
                match result.0 {
                    gui::VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::VendorResult::NoResponse => {},
                    gui::VendorResult::Sell => {
                        let price = self.ecs.read_storage::<Item>().get(result.1.unwrap()).unwrap().base_value * 0.8;
                        self.ecs.write_storage::<Pools>().get_mut(*self.ecs.fetch::<Entity>()).unwrap().gold += price;
                        self.ecs.delete_entity(result.1.unwrap()).expect("Unable to delete");
                    },
                    gui::VendorResult::Buy => {
                        let tag = result.2.unwrap();
                        let price = result.3.unwrap();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*self.ecs.fetch::<Entity>()).unwrap();
                        if player_pools.gold >= price {
                            player_pools.gold -= price;
                            std::mem::drop(pools);
                            let player_entity = *self.ecs.fetch::<Entity>();
                            crate::raws::spawn_named_item(&raws::RAWS.lock().unwrap(),
                                &mut self.ecs, &tag, raws::SpawnType::Carried { by: player_entity });
                        }
                    },
                    gui::VendorResult::BuyMode => newrunstate = RunState::ShowVendor { vendor, mode: VendorMode::Buy },
                    gui::VendorResult::SellMode => newrunstate = RunState::ShowVendor { vendor, mode: VendorMode::Sell },
                }
            } RunState::SaveGame => {
                saveload_sys::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::LoadGame };
            } RunState::NextLevel => {
                self.goto_level(1);
                newrunstate = RunState::PreRun;
            } RunState::PreviousLevel => {
                self.goto_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            } RunState::TownPortal => {
                spawner::spawn_town_portal(&mut self.ecs);
                let map_depth = self.ecs.fetch::<Map>().depth;
                let dest_offset = 0 - (map_depth-1);
                self.goto_level(dest_offset);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            } RunState::ShowRemoveEquipment => {
                let result = gui::remove_equipment_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveEquipment>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToRemoveEquipment { item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    },
                }
            } RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {},
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame };
                    },
                }
            } RunState::MagicMapReveal {row} => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0 .. map.width {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                };
                if row == map.height-1 {
                    newrunstate = RunState::Ticking;
                } else {
                    newrunstate = RunState::MagicMapReveal { row: row+1 };
                }
            } RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                }
                ctx.cls();
                if self.mapgen_index < self.mapgen_history.len() {
                    camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                }

                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        newrunstate = self.mapgen_next_state.unwrap();
                    }
                }
            } RunState::TeleportingToOtherLevel { x, y, depth } => {
                self.goto_level(depth-1);
                let player_entity = self.ecs.fetch::<Entity>();
                if let Some(pos) = self.ecs.write_storage::<Position>().get_mut(*player_entity) {
                    pos.x = x;
                    pos.y = y;
                }
                let mut ppos = self.ecs.fetch_mut::<rltk::Point>();
                ppos.x = x;
                ppos.y = y;
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
        }
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter= newrunstate;
        }
        dmg_sys::delete_the_dead(&mut self.ecs);
    }
}

impl State {
    fn goto_level (&mut self, offset:i32) {
        freeze_level_entities(&mut self.ecs);

        /* Build a new map and place the player */
        let current_depth = self.ecs.fetch::<Map>().depth;
        self.generate_world_map(current_depth+offset, offset);

        /* Notify the player */
        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog.entries.push("You change level.".to_string());
    }

    fn game_over_cleanup (&mut self) {
        /* Delete Everything */
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        };
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        };

        { /* Spawn new player */
            let player_entity = spawner::player(&mut self.ecs, 0, 0);
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity; 
        }
        self.ecs.insert(map::MasterDungeonMap::new());
        self.generate_world_map(1, 0);
    }

    fn generate_world_map (&mut self, new_depth:i32, offset:i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = map::level_transition(&mut self.ecs, new_depth, offset);
        if let Some(history) = map_building_info {
            self.mapgen_history = history;
        } else {
            map::thaw_level_entities(&mut self.ecs);
        }
    }
}
/* }}} */
// -- Main -- {{{
fn main () -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple(80,60)
        .unwrap().with_title("Shadorogue").build()?;
    /* RETRO Feel: context needs `mut` */
    // context.with_post_scanlines(true);
    let mut gs = State { 
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

    gs.ecs.insert(map::MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleeWeapon>();
    gs.ecs.register::<Wearable>();
    gs.ecs.register::<WantsToRemoveEquipment>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<NaturalAttackDefense>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<EquipmentChanged>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<TownPortal>();
    gs.ecs.register::<TeleportTo>();
    gs.ecs.register::<ApplyMove>();
    gs.ecs.register::<ApplyTeleport>();
    gs.ecs.register::<MagicItem>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    raws::load_raws();

    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    let player_entity = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MapGeneration{});
    gs.ecs.insert(gamelog::GameLog { entries : vec!["Welcome to Roguelike".to_string()] });
    gs.ecs.insert(particle_sys::ParticleBuilder::new());
    gs.ecs.insert(rex_assets::RexAssets::new());

    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs)
}
/* }}} */
/* -------------------------------------------------------------------------- */
