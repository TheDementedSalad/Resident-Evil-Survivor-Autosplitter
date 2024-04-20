#![no_std]
// #![feature(type_alias_impl_trait, const_async_blocks)]
#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::style,
    clippy::undocumented_unsafe_blocks,
    rust_2018_idioms
)]

use asr::{
    emulator::ps1::Emulator,
    future::{next_tick, retry},
    time::Duration,
    time_util::frame_count,
    timer::{self, TimerState},
    watcher::Watcher,
    settings::Gui
};

asr::panic_handler!();
asr::async_main!(stable);

//Creates a macro which let's us use unwrap_or
macro_rules! unwrap_or {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return false,
        }
    };
}

// Creates a struct name Door which will hold information
impl Door {
    // Creates a public constant function, containing the stage_id, current_room_id and old_room_id
    pub const fn new(stage_id: u8, current_room_id: u8, old_room_id: u8 ) -> Self {
        Self {
            stage_id,
            current_room_id,
            old_room_id,
        }
    }
}

// Creates a constant struct, which references the struct "Door" and stores a list of tuples containing information about door splits
const DOORS: [Door; 90] = [
    Door::new(7, 0, 4),
    Door::new(8, 1, 0),
    Door::new(8, 2, 1),
    Door::new(8, 3, 2),
    Door::new(8, 2, 3),
    Door::new(8, 1, 2),
    Door::new(8, 0, 1),
    Door::new(7, 1, 0), // Street Entrance
    Door::new(7, 2, 1),
    Door::new(7, 3, 2),
    Door::new(14, 0, 3),
    Door::new(14, 1, 0),
    Door::new(14, 0, 1),
    Door::new(14, 2, 0),
    Door::new(14, 3, 2),
    Door::new(14, 4, 3),
    Door::new(3, 14, 4), // Sewer Entrance
    Door::new(3, 15, 14),
    Door::new(3, 16, 15),
    Door::new(3, 0, 16),
    Door::new(3, 2, 0),
    Door::new(3, 3, 2),
    Door::new(3, 4, 3),
    Door::new(3, 3, 4),
    Door::new(3, 2, 3),
    Door::new(3, 5, 2),
    Door::new(3, 1, 5),
    Door::new(3, 8, 1),
    Door::new(3, 7, 8),
    Door::new(3, 8, 7),
    Door::new(3, 9, 8),
    Door::new(3, 10, 9),
    Door::new(10, 0, 10),
    Door::new(10, 1, 0),
    Door::new(10, 2, 1),
    Door::new(10, 3, 2),
    Door::new(10, 4, 3),
    Door::new(11, 0, 4), //Umbrella HQ Entrance
    Door::new(11, 1, 0),
    Door::new(11, 3, 1),
    Door::new(11, 4, 3),
    Door::new(11, 3, 4),
    Door::new(11, 5, 3),
    Door::new(11, 6, 5),
    Door::new(11, 7, 6),
    Door::new(11, 2, 7), //Defeat Mr X 1
    Door::new(11, 8, 2),
    Door::new(16, 0, 8),
    Door::new(16, 1, 0),
    Door::new(16, 2, 1),
    Door::new(16, 3, 2),
    Door::new(16, 4, 3),
    Door::new(16, 6, 4), //Meet Lily
    Door::new(16, 4, 6),
    Door::new(16, 3, 4),
    Door::new(16, 7, 3),
    Door::new(16, 8, 7),
    Door::new(16, 7, 8),
    Door::new(16, 9, 7),
    Door::new(16, 10, 9), //Cable Car RNG end
    Door::new(16, 11, 10),
    Door::new(5, 0, 11),
    Door::new(6, 0, 11),
    Door::new(2, 1, 0),
    Door::new(2, 2, 1),
    Door::new(2, 3, 2), //Enter Facility
    Door::new(2, 4, 3),
    Door::new(2, 5, 4),
    Door::new(2, 6, 5),
    Door::new(2, 8, 6),
    Door::new(2, 9, 8),
    Door::new(2, 8, 9),
    Door::new(2, 6, 8),
    Door::new(2, 5, 6),
    Door::new(2, 7, 5),
    Door::new(2, 5, 7),
    Door::new(2, 6, 5),
    Door::new(2, 10, 6),
    Door::new(2, 11, 10),
    Door::new(2, 12, 11), //Mr X 2
    Door::new(2, 13, 12), // Save Lott
    Door::new(2, 15, 13),
    Door::new(2, 13, 15),
    Door::new(2, 14, 13),
    Door::new(2, 13, 14),
    Door::new(2, 15, 13), //Hypnos 1
    Door::new(2, 16, 15),
    Door::new(17, 0, 16),
    Door::new(17, 1, 0),
    Door::new(17, 2, 1),
];

#[derive(Eq, PartialEq)]
struct Door {
    pub stage_id: u8,
    pub current_room_id: u8,
    pub old_room_id: u8,
}

async fn main() {
    let settings = Settings::register();

    loop {
        // Hook to the target process
        let mut emulator = retry(|| Emulator::attach()).await;
        let mut watchers = Watchers::default();
        let offsets = Offsets::new();
        let mut settings = Settings::register();

        loop {
            settings.update();

            if !emulator.is_open() {
                break;
            }

            if emulator.update() {
                // Splitting logic. Adapted from OG LiveSplit:
                // Order of execution
                // 1. update() will always be run first. There are no conditions on the execution of this action.
                // 2. If the timer is currently either running or paused, then the isLoading, gameTime, and reset actions will be run.
                // 3. If reset does not return true, then the split action will be run.
                // 4. If the timer is currently not running (and not paused), then the start action will be run.
                update_loop(&emulator, &offsets, &mut watchers);

                let timer_state = timer::state();
                if timer_state == TimerState::Running || timer_state == TimerState::Paused {
                    if let Some(is_loading) = is_loading(&watchers, &settings) {
                        if is_loading {
                            timer::pause_game_time()
                        } else {
                            timer::resume_game_time()
                        }
                    }

                    if let Some(game_time) = game_time(&watchers, &settings) {
                        timer::set_game_time(game_time)
                    }

                    if reset(&watchers, &settings) {
                        timer::reset()
                    } else if split(&watchers, &settings) {
                        timer::split()
                    }
                }

                if timer::state() == TimerState::NotRunning && start(&watchers, &settings) {
                    timer::start();
                    timer::pause_game_time();

                    if let Some(is_loading) = is_loading(&watchers, &settings) {
                        if is_loading {
                            timer::pause_game_time()
                        } else {
                            timer::resume_game_time()
                        }
                    }
                }
            }
            next_tick().await;
        }
    }
}

// This is where we will create our settings
#[derive(Gui)]
struct Settings {
    #[default = true]
    /// ---------- Start Conditions Below ----------
    _condit: bool,

    #[default = false]
    /// IGT Start - Starts when IGT starts
    igtstart: bool,

    #[default = false]
    /// RTA Start - Starts when selecting difficulty or loading save
    rtastart: bool,

    #[default = true]
    /// ---------- End Split Below ----------
    _ending: bool,

    #[default = false]
    /// IGT End - Splits when IGT ends at final credits
    igtend: bool,

    #[default = false]
    /// RTA End - Splits when final boss hits 0 HP
    rtaend: bool,

    #[default = true]
    /// ---------- Door Splits Below ----------
    _doors: bool,

    #[default = false]
    /// Door splits - Will split on every room
    door_split: bool,

    #[default = true]
    /// ---------- Item Splits Below ----------
    _items: bool,

    #[default = false]
    /// Rusted Key
    rust: bool,

    #[default = false]
    /// Manager's Key
    manage: bool,

    #[default = false]
    /// Cracked Key
    crack: bool,

    #[default = false]
    /// Arcade Key
    arcade: bool,

    #[default = false]
    /// Manhole Opener
    manhole: bool,

    #[default = false]
    /// Prison Cell Key
    prison: bool,

    #[default = false]
    /// Rope
    rope: bool,

    #[default = false]
    /// Shotgun
    shotgun: bool,

    #[default = false]
    /// Card Key
    cardkey: bool,

    #[default = false]
    /// Grenade Gun
    nadegun: bool,

    #[default = false]
    /// Magnum
    magnum: bool,

    #[default = false]
    /// Activation Disk
    disk: bool,

    #[default = false]
    /// ID Card
    idcard: bool,

    #[default = false]
    /// Master Key
    master: bool,

    #[default = false]
    /// ---------- Area Splits Below ----------
    _areas: bool,

    #[default = false]
    /// Enter street after restaurant
    street: bool,

    #[default = false]
    /// Enter the sewers
    sewer: bool,

    #[default = false]
    /// Reach Umbrella HQ
    umbhq: bool,

    #[default = false]
    /// Defeat Mr. X at the elevator
    mrx1: bool,

    #[default = false]
    /// Meet Lily at the house
    lily: bool,

    #[default = false]
    /// End of cable car RNG section
    cable: bool,

    #[default = false]
    /// Enter the facility
    facil: bool,

    #[default = false]
    /// Mr. X on the long bridge
    mrx2: bool,

    #[default = false]
    /// Save Lott from the hunter
    huntlott: bool,

    #[default = false]
    /// Defeat Hypnos 1
    hypno1: bool,

    #[default = false]
    /// Defeat Hypnos 2
    hypno2: bool,
}

// Defines the watcher type of
#[derive(Default)]
struct Watchers {
    hp: Watcher<u16>,
    map_id: Watcher<u8>,
    stage_id: Watcher<u8>,
    item_id: Watcher<u8>,
    gamestate: Watcher<u32>,
    startbuff: Watcher<u8>,
    inventory: Watcher<[u8; 224]>,

    //Boss HP Related
    hypno1: Watcher<i16>,
    hypno2: Watcher<i16>,
    hypno3: Watcher<i16>,
    
    //IGT Related
    igt: Watcher<Duration>,
    accumulated_igt: Duration,
    buffer_igt: Duration,
}

struct Offsets {
    gamecode_ntsc: u32,
    
    hp: u32,
    igt: u32,
    map_id: u32,
    stage_id: u32,
    item_id: u32,
    gamestate: u32,
    startbuff: u32,

    //Boss HP's
    hypno1: u32,
    hypno2: u32,
    hypno3: u32,
}

// Offsets of data, relative to the beginning of the games VRAM
impl Offsets {
    fn new() -> Self {
        Self {
            gamecode_ntsc: 0x940C,

            hp: 0x800A8974,                // Character HP
            igt: 0x80063BE4,               // In Game Time
            map_id: 0x800B4130,            // Current Room's ID
            stage_id: 0x800B4EB0,          // Current Stage ID
            item_id: 0x800AF890,           // 1st item in array
            gamestate: 0x801FF998,        // gamestate
            startbuff: 0x8007FA9D,         // check to stop timer starting in demo

            //Boss HP's
            hypno1: 0x801EE41E,
            hypno2: 0x801F104A,           // final boss phase 1
            hypno3: 0x801F190E,           // final boss phase 2
        }
    }
}

fn update_loop(game: &Emulator, offsets: &Offsets, watchers: &mut Watchers) {
    match &game.read::<[u8; 11]>(offsets.gamecode_ntsc).unwrap_or_default()
    {
         // The gamecodes here ensure you're running the right game
        b"SLPS_025.53" => {
            // IGT Watcher that calculates the time
            watchers.igt.update_infallible(frame_count::<30>(
                game.read::<u32>(offsets.igt).unwrap_or_default() as _,
            ));
            // Inventory Watcher that creates an array of 224 items
            watchers.inventory.update_infallible(
                game.read::<[[u8; 8]; 224]>(offsets.item_id).unwrap_or([[0; 8]; 224]).map(|[item, _, _, _, _, _, _, _]| item),
            );

            // Boss HP Watchers
            watchers.hypno1.update(game.read::<i16>(offsets.hypno1).ok());
            watchers.hypno2.update(game.read::<i16>(offsets.hypno2).ok());
            watchers.hypno3.update(game.read::<i16>(offsets.hypno3).ok());

            // Misc Watchers
            watchers.hp.update(game.read::<u16>(offsets.hp).ok());
            watchers.map_id.update(game.read::<u8>(offsets.map_id).ok());
            watchers.stage_id.update(game.read::<u8>(offsets.stage_id).ok());
            watchers.item_id.update(game.read::<u8>(offsets.item_id).ok());
            watchers.gamestate.update(game.read::<u32>(offsets.gamestate).ok());
            watchers.startbuff.update(game.read::<u8>(offsets.startbuff).ok());
        }    
        _ => {
            // If the emulator is loading the wrong game, the watchers will update to their default state
            watchers.hp.update_infallible(u16::default());
            watchers.igt.update_infallible(Duration::default());
            watchers.map_id.update_infallible(u8::default());
            watchers.stage_id.update_infallible(u8::default());
            watchers.item_id.update_infallible(u8::default());
            watchers.startbuff.update_infallible(u8::default());
            watchers.gamestate.update_infallible(u32::default());

            //Bosses
            watchers.hypno1.update_infallible(i16::default());
            watchers.hypno2.update_infallible(i16::default());
            watchers.hypno3.update_infallible(i16::default());
        }
    };


    // Reset the buffer IGT variables when the timer is stopped
    if timer::state() == TimerState::NotRunning {
        watchers.accumulated_igt = Duration::ZERO;
        watchers.buffer_igt = Duration::ZERO;
    }

    if let Some(igt) = &watchers.igt.pair {
        if igt.old > igt.current {
            watchers.accumulated_igt += igt.old - watchers.buffer_igt;
            watchers.buffer_igt = igt.current;
        }
    }
}

// If the setting "start" is not selected, nothing will happen
// Checks to see if the current IGT > 0 and the old IGT == 0
fn start(watchers: &Watchers, settings: &Settings) -> bool {
    if !settings.igtstart && !settings.rtastart {
        return false;
    }
    settings.igtstart && watchers.igt.pair.is_some_and(|pair| pair.changed_from(&Duration::ZERO)) 
        && watchers.map_id.pair.is_some_and(|pair| pair.current == 4)
        && watchers.stage_id.pair.is_some_and(|pair| pair.current == 7)
        || settings.rtastart && watchers.gamestate.pair.is_some_and(|pair| pair.current == 2147932080
            && watchers.startbuff.pair.is_some_and(|pair| pair.current == 1))


}

// This is where the conditions for your split settings will go
fn split(watchers: &Watchers, settings: &Settings) -> bool {

    let stage_id= unwrap_or!(watchers.stage_id.pair).current;
 
    let room_id = unwrap_or!(watchers.map_id.pair);

    if settings.door_split && (DOORS.contains(&Door::new(stage_id, room_id.current, room_id.old)) 
        || watchers.stage_id.pair.is_some_and(|i| i.changed_from_to(&5, &2)) 
            && watchers.map_id.pair.is_some_and(|i| i.current == 0))
    {
        true
    }
    else if settings.igtend && watchers.gamestate.pair.is_some_and(|i| i.changed_from_to(&2147638888, &2147971644))
        || settings.rtaend && watchers.hypno2.pair.is_some_and(|i| i.current <= 0)
            && watchers.hypno3.pair.is_some_and(|i| i.current <= 0 && i.old >= 1)
            && watchers.hypno3.pair.is_some_and(|i| i.current <= 0)
            && watchers.map_id.pair.is_some_and(|i| i.current == 2) 
            && watchers.stage_id.pair.is_some_and(|i| i.current == 17)

        || settings.street && watchers.stage_id.pair.is_some_and(|i| i.current == 7)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&0, &1))
        || settings.sewer && watchers.stage_id.pair.is_some_and(|i| i.current == 3)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&4, &14))
        || settings.umbhq && watchers.stage_id.pair.is_some_and(|i| i.current == 11)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&4, &0))
        || settings.mrx1 && watchers.stage_id.pair.is_some_and(|i| i.current == 11)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&7, &2))
        || settings.lily && watchers.stage_id.pair.is_some_and(|i| i.current == 16)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&4, &6))
        || settings.cable && watchers.stage_id.pair.is_some_and(|i| i.current == 16)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&9, &10))
        || settings.facil && watchers.stage_id.pair.is_some_and(|i| i.current == 2)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&2, &3))
        || settings.mrx2 && watchers.stage_id.pair.is_some_and(|i| i.current == 2)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&11, &12))
        || settings.huntlott && watchers.stage_id.pair.is_some_and(|i| i.current == 2)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&12, &13))
        || settings.hypno1 && watchers.stage_id.pair.is_some_and(|i| i.current == 2)
            && watchers.map_id.pair.is_some_and(|i| i.changed_from_to(&13, &15))
            && watchers.hypno1.pair.is_some_and(|i| i.current <= 0)
        || settings.hypno2 && watchers.stage_id.pair.is_some_and(|i| i.current == 17)
            && watchers.map_id.pair.is_some_and(|i| i.current == 2)
            && watchers.hypno2.pair.is_some_and(|i| i.current <= 0 && i.old >= 1)
            && watchers.hypno3.pair.is_some_and(|i| i.current == 900)
    {
        true
    }
    else {
        watchers.inventory.pair.is_some_and(|inventory| {
        settings.rust && inventory.check(|arr| arr.contains(&36))
        || settings.manage && inventory.check(|arr| arr.contains(&37))
        || settings.crack && inventory.check(|arr| arr.contains(&44))
        || settings.arcade && inventory.check(|arr| arr.contains(&45))
        || settings.manhole && inventory.check(|arr| arr.contains(&48))
        || settings.prison && inventory.check(|arr| arr.contains(&41))
        || settings.rope && inventory.check(|arr| arr.contains(&43))
        || settings.shotgun && inventory.check(|arr| arr.contains(&4))
        || settings.cardkey && inventory.check(|arr| arr.contains(&47))
        || settings.nadegun && inventory.check(|arr| arr.contains(&8))
        || settings.magnum && inventory.check(|arr| arr.contains(&5))
        || settings.disk && inventory.check(|arr| arr.contains(&52))
        || settings.idcard && inventory.check(|arr| arr.contains(&54))
        || settings.master && inventory.check(|arr| arr.contains(&53))
        })
    }
}

fn reset(_watchers: &Watchers, _settings: &Settings) -> bool {
    false
}

// Some(true) is equivelant to "return true"
fn is_loading(_watchers: &Watchers, settings: &Settings) -> Option<bool> {
   settings.igtstart.then_some(true)
}

// Returns the current game time, which is calculated up in the update loop
fn game_time(watchers: &Watchers, settings: &Settings) -> Option<Duration> {
    settings.igtstart.then_some(watchers.igt.pair?.current + watchers.accumulated_igt - watchers.buffer_igt)
}
