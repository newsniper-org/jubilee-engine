use wasm_bindgen::prelude::*;
use rhai::{Engine, Map, Scope};
use serde::{Serialize, Deserialize};
use std::{cmp::min, collections::HashMap, ops::{Add, AddAssign, Sub, SubAssign}};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tile {
    pub name: String,
    #[serde(rename = "type")]
    pub tile_type: String,
    #[serde(default)]
    pub price: i64,
    #[serde(default)]
    pub amount: i64,
    #[serde(default)]
    pub is_coastal: bool,
    #[serde(default)]
    pub is_megacity: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChanceCard {
    pub title: String,
    pub descriptoin: String,
    pub instruction: String,
}

#[derive(Serialize, Clone, Debug)]
#[repr(u8)]
pub enum EducationStatus {
    NotYet = 0u8, Undergraduated = 1u8, Graduated = 2u8
}

impl EducationStatus {
    pub(crate) fn educate(&mut self) {
        *self = match *self {
            Self::NotYet => Self::Undergraduated,
            Self::Undergraduated => Self::Graduated,
            Self::Graduated => Self::Graduated
        };
    }
}

#[wasm_bindgen]
#[derive(Serialize, Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TicketCount {
    pub free_hospital: u32,
    pub free_property: u32,
    pub double_lotto: u32,
    pub no_tax: u32,
    pub release_from_jail: u32,
    pub bonus: u32
}

impl TicketCount {
    #[inline(always)]
    fn sub_nonnegative(lhs: u32, rhs: u32) -> u32 {
        if lhs < rhs {
            0
        } else {
            lhs - rhs
        }
    }

    #[inline(always)]
    pub fn zero() -> Self {
        Self {
            free_hospital: 0,
            free_property: 0,
            double_lotto: 0,
            no_tax: 0,
            release_from_jail: 0,
            bonus: 0
        }
    }

    pub fn get_one_ticket(kind: &str) -> Self {
        match kind {
            "FreeHospital" => {
                Self {
                    free_hospital: 1,
                    ..Default::default()
                }
            },
            "FreeProperty" => {
                Self {
                    free_property: 1,
                    ..Default::default()
                }
            },
            "DoubleLotto" => {
                Self {
                    double_lotto: 1,
                    ..Default::default()
                }
            },
            "NoTax" => {
                Self {
                    no_tax: 1,
                    ..Default::default()
                }
            },
            "ReleaseFromJail" => {
                Self {
                    release_from_jail: 1,
                    ..Default::default()
                }
            },
            "Bonus" => {
                Self {
                    bonus: 1,
                    ..Default::default()
                }
            },
            _ => {
                Self::default()
            }
        }
    }
}

impl Add for TicketCount {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            free_hospital: self.free_hospital + rhs.free_hospital,
            free_property: self.free_property + rhs.free_property,
            double_lotto: self.double_lotto + rhs.double_lotto,
            no_tax: self.no_tax + rhs.no_tax,
            release_from_jail: self.release_from_jail + rhs.release_from_jail,
            bonus: self.bonus + rhs.bonus
        }
    }
}

impl AddAssign for TicketCount {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for TicketCount {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            free_hospital: Self::sub_nonnegative(self.free_hospital, rhs.free_hospital),
            free_property: Self::sub_nonnegative(self.free_property, rhs.free_property),
            double_lotto: Self::sub_nonnegative(self.double_lotto, rhs.double_lotto),
            no_tax: Self::sub_nonnegative(self.no_tax, rhs.no_tax),
            release_from_jail: Self::sub_nonnegative(self.release_from_jail, rhs.release_from_jail),
            bonus: Self::sub_nonnegative(self.bonus, rhs.bonus)
        }
    }
}

impl SubAssign for TicketCount {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Default for TicketCount {
    #[inline(always)]
    fn default() -> Self {
        Self::zero()
    }
}


#[derive(Serialize, Clone, Debug)]
pub struct Player {
    pub id: u32,
    pub position: u32,
    pub money: i64,
    pub remaining_loans: Vec<(u32, i64, u32)>,
    pub education_status: EducationStatus,
    pub cycles: u32,
    pub remaining_jail_turns: u32,
    pub tickets_count: TicketCount,
}

#[derive(Serialize, Clone, Debug)]
pub struct GameState {
    board: Vec<Tile>,
    chance_cards_inventory: HashMap<String, ChanceCard>,
    players: Vec<Player>,
    properties: HashMap<String, (u32,u32)>,
    log: Vec<String>,
    current_turn_idx: usize,
    government_income: i64,
    dice_double: bool,
    pandemic_counter: usize,
    catastrophe_counter: usize,
    consts: HashMap<String, u32>,
    pending_ticket: TicketCount,
    luck_test_cache: i64,
}

#[wasm_bindgen]
pub enum GameSituation {
    InAction,
    PendingBuyResponse,
    PendingFinancialCrisisResponse,
    PendingRollResponse,
    PendingLuckTestResponse,
    PendingUseTicketResponse,
    PendingTryToJailbreakResponse,
    PendingGetRandomChanceCardResponse,
    PendingCheckChanceCardResponse,
    EndTurn,
    EndGame
}

#[wasm_bindgen]
pub struct GameEngine {
    pub(crate) engine: Engine,
    pub(crate)state: GameState,
    salary: i64,
    building_cost: i64,
    now: GameSituation,
    pending_chance_card_id: Option<String>
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DicePair(u16, u16);

impl Serialize for DicePair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let concatenated = (self.0 as u32) << 16 | (self.1 as u32);
        concatenated.serialize::<S>(serializer)
    }
}

impl<'de> Deserialize<'de> for DicePair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let result = u32::deserialize::<D>(deserializer);
        match result {
            Ok(concatenated) => {
                Ok(Self((concatenated >> 16) as u16, concatenated as u16))
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

#[wasm_bindgen]
impl DicePair {
    pub fn is_double(&self) -> bool {
        self.0 == self.1
    }
}


fn map_pair<T, R, F>(pair: (T, T), f: F) -> (R, R)
where F: Fn(T) -> R {
    (f(pair.0), f(pair.1))
}

fn map_partition<T, IT, R, F, IR>(partition: (IT, IT), f: F) -> (IR, IR)
where
    F: Fn(T) -> R,
    IT: Iterator<Item = T> + Sized,
    IR: FromIterator<R> + Sized {
    let mapped = map_pair(partition,|element| {
        element.map(|item| f(item)).collect::<IR>()
    });
    mapped
}



#[wasm_bindgen]
impl GameEngine {
    fn round(x: i64, n: i64) -> i64 {
        let rem = x % n;
        if (2 * rem) >= n {
            x - rem + n
        } else {
            x - rem
        }
    }

    #[wasm_bindgen(constructor)]
    pub fn new(board_json: &str, chance_cards_json: &str, consts_json: &str, players_count: usize, initial_money: i64, salary: i64, building_cost: i64) -> Result<GameEngine, String> {
        let board: Vec<Tile> = serde_json::from_str(board_json).map_err(|e| e.to_string())?;
        let chance_cards_inventory: HashMap<String, ChanceCard> = serde_json::from_str(chance_cards_json).map_err(|e| e.to_string())?;
        let consts: HashMap<String, u32> = serde_json::from_str(consts_json).map_err(|e| e.to_string())?;
        let state = GameState {
            board,
            chance_cards_inventory,
            players: (0..players_count).map(|i| Player { id: (i+1) as u32, position: 0, money: initial_money, remaining_loans: Vec::new(), education_status: EducationStatus::NotYet, cycles: 0, remaining_jail_turns: 0, tickets_count: TicketCount::default() }).collect(),
            properties: HashMap::new(),
            log: vec!["Game started!".into()],
            current_turn_idx: 0,
            government_income: 0,
            dice_double: false,
            pandemic_counter: 0,
            catastrophe_counter: 0,
            consts,
            pending_ticket: TicketCount::zero(),
            luck_test_cache: -1,
        };
        let mut engine = Engine::new();

        // Rhai가 Rust 객체를 사용할 수 있도록 등록
        engine.register_type_with_name::<Tile>("Tile");
        engine.register_get("name", |t: &mut Tile| t.name.clone());
        engine.register_get("type", |t: &mut Tile| t.tile_type.clone());
        engine.register_get("price", |t: &mut Tile| t.price);
        engine.register_get("amount", |t: &mut Tile| t.amount);
        engine.register_get("is_coastal", |t: &mut Tile| t.is_coastal);
        engine.register_get("is_megacity", |t: &mut Tile| t.is_megacity);

        engine.register_type_with_name::<TicketCount>("TicketCount");
        engine.register_get("free_hospital", |tc: &mut TicketCount| tc.free_hospital);
        engine.register_get("free_property", |tc: &mut TicketCount| tc.free_property);
        engine.register_get("no_tax", |tc: &mut TicketCount| tc.no_tax);
        engine.register_get("release_from_jail", |tc: &mut TicketCount| tc.release_from_jail);
        engine.register_get("bonus", |tc: &mut TicketCount| tc.bonus);

        // 플레이어 수 확인을 위한 API
        let state_clone = state.clone();
        engine.register_fn("get_player_count", move || -> i64 {
            state_clone.players.len() as i64
        });

        // 10만 단위 반올림을 위한 API
        engine.register_fn("round100000", |x: i64| -> i64 {
            Self::round(x, 100000)
        });

        engine.register_fn("find_next_tile_of_type", move |current_pos: u32, tile_type: String| -> u32 {
            // current_pos 다음부터 순환하며 tile_type을 가진 첫 타일의 인덱스를 찾아 반환
            let found = state_clone.board.clone().into_iter().enumerate().filter_map( move |(i, tile)| {
                if ((i as u32) != current_pos) && (tile.tile_type == tile_type) {
                    Some(i)
                } else { None }
            }).collect::<Vec<_>>();
            let (a, b): (Vec<_>, Vec<_>) = found.into_iter().partition(|&n| {
                (n as u32) > current_pos
            });
            if a.is_empty() {
                if b.is_empty() {
                    current_pos
                } else {
                    b[0] as u32
                }
            } else {
                a[0] as u32
            }
        });

        let board_clone = state.board.clone();
        engine.register_fn("get_coastal_cities", move || -> Vec<String> {
            let coastal_cities = Self::get_coastal_cities(&board_clone);
            coastal_cities
        });

        Ok(Self {
            engine, state, salary, building_cost,
            pending_chance_card_id: None,
            now: GameSituation::PendingRollResponse
        })
    }

    fn get_coastal_cities(board: &Vec<Tile>) -> Vec<String> {
        // board_iter.iter().filter(|tile| )
        board.iter().filter_map(|tile| {
            if tile.is_coastal {
                Some(tile.name.clone())
            } else { None }
        }).collect()
    }

    fn get_owned_properties(props: &HashMap<String, (u32, u32)>, player_id: u32) -> (HashMap<String, u32>, HashMap<String, u32>) {
        let partition: (Vec<_>, Vec<_>) = props.iter().partition(|&(_, (owner_id, _))| {
            *owner_id == player_id
        });
        let partition_iterators = (partition.0.iter(), partition.1.iter());
        let mapped_partition = map_partition::<&(&String, &(u32, u32)),_,(String, u32),_,HashMap<_, _>>(partition_iterators, |&(name, &(_, owned_amount))| {
            (name.clone(), owned_amount)
        });
        mapped_partition
    }


    fn try_run_turn_script(&mut self, script_action: &str, dices: Option<DicePair>, script_cycle: &str, to_use_ticket: i64) -> Result<(), String> {
        self.now = GameSituation::InAction;
        let mut scope = Scope::new();
        let player_index = self.state.current_turn_idx;
        let player = self.state.players[player_index].clone();
        
        // scope에 현재 플레이어의 id를 넘김
        scope.push("player_id", player.id); 

        let dices = dices.unwrap_or(DicePair(0, 0));

        let old_pos = player.position.clone();
        // 이동 후 위치 계산
        let new_pos = (player.position + (dices.0 + dices.1) as u32) % self.state.board.len() as u32;
        let tile = self.state.board[new_pos as usize].clone();
        let is_owned = self.state.properties.contains_key(&tile.name);
        let (owner_id, owned_amount) = if let Some(&(owner_id, owned_amount)) = self.state.properties.get(&tile.name) {
            (Some(owner_id), Some(owned_amount))
        } else {
            (None, None)
        };

        // 주사위가 더블일 때
        if dices != DicePair(0, 0) {
            if dices.0 == dices.1 {
                self.state.dice_double = true;
            } else {
                self.state.dice_double = false;
            }
        }
        

        // 한 바퀴를 채웠으면 
        if old_pos >= new_pos && dices != DicePair(0, 0) {
            self.trigger_cycle(script_cycle)?;
        }

        scope.push("tile", tile);
        scope.push("is_owned", is_owned);
        scope.push("owner_id", owner_id);
        scope.push("owned_amount", owned_amount);
        scope.push("building_cost", self.building_cost);
        scope.push_constant("MAX_BUILDINGS", if let Some(&max_buildings) = self.state.consts.get("MAX_BUILDINGS") && max_buildings > 0 {
            max_buildings
        } else {
            1
        });

        scope.push("to_use_ticket", to_use_ticket);
        
        let tickets = self.state.players[player_index].tickets_count.clone();
        scope.push("tickets", tickets);

        let result: Map = self.engine.eval_with_scope(&mut scope, script_action).map_err(|e| e.to_string())?;

        let action_type = result["type"].clone().into_string().unwrap();
        match action_type.as_str() {
            "PromptBuy" => {
                let name = result["tile_name"].clone().into_string().unwrap();
                let price = result["price"].clone().as_int().unwrap();
                self.state.log.push(format!("Landed on {}'{}'.", if let Some(_) = owner_id { "" } else { "unowned " }, name));
                // 구매 로직
                let player_mut = &mut self.state.players[player_index];
                
                let free_flag = result["free_flag"].clone().as_bool().unwrap_or(false);
                let ticket_flag = result["ticket_flag"].clone().as_bool().unwrap_or(false);
                if !free_flag && !ticket_flag {
                    player_mut.money -= price;
                }

                if player_mut.money >= self.building_cost {
                    self.state.log.push(format!("Buy {} building for ${}?", if let Some(_) = owner_id { "one more" } else { "a" },self.building_cost));
                    self.now = GameSituation::PendingBuyResponse;                  
                } else {
                    self.state.log.push("Not enough money to buy.".into());
                }
            },
            "PayTax" => {
                let amount = result["amount"].clone().as_int().unwrap();
                self.state.players[player_index].money -= amount;
                self.state.government_income += amount;
                self.state.log.push(format!("Player {} Paid ${} in taxes.", self.state.players[player_index].id, amount));

                if self.state.players[player_index].money < 0 {
                    self.prompt_financial_crisis();
                }
            },
            "Imprison" => {
                self.state.log.push("Imprisoned!".into());
                self.now = GameSituation::EndTurn;
            },
            "WarpToPosition" => {
                let dest = result["position"].clone().as_int().unwrap() as u32;
                self.state.players[player_index].position = dest;
                self.state.log.push(format!("Warped to {}!", self.state.board[dest as usize].name));
                self.now = GameSituation::EndTurn;
                return Ok(()); // 이동 로직을 건너뛰기 위해 여기서 종료
            },
            "PayTo" => {
                let government_amount = if let Ok(amount) = result["gov_amount"].clone().as_int() {
                    Some(amount)
                } else { None };
                let market_amount = if let Ok(amount) = result["market_amount"].clone().as_int() {
                    Some(amount)
                } else { None };
                let to_player = if let Ok(amount) = result["player_amount"].clone().as_int() && let Ok(pid) = result["to_player_id"].clone().as_int() {
                    Some((amount, pid as u32))
                } else { None };
                let payer_id = self.state.players[player_index].id;

                let message = result["message"].clone().into_string().unwrap();
                self.state.log.push(message);

                if let Some(amount) = government_amount {
                    self.state.government_income += amount;
                    self.state.players[player_index].money -= amount;
                    self.state.log.push(format!("\tPlayer {} Paid ${} to the government.", payer_id, amount));
                }

                if let Some(amount) = market_amount {
                    self.state.players[player_index].money -= amount;
                    self.state.log.push(format!("\tPlayer {} Paid ${} to the market.", payer_id, amount));
                }

                if let Some((amount, pid)) = to_player {
                    let to_player = self.state.players.iter_mut().find(|player| player.id == pid);
                    if let Some(to_player_mut) = to_player {
                        to_player_mut.money += amount;
                        self.state.players[player_index].money -= amount;
                        self.state.log.push(format!("\tPlayer {} Paid ${} to Player {}.", payer_id, amount, pid));
                    }
                }

                if self.state.players[player_index].money < 0 {
                    self.prompt_financial_crisis();
                }
            },
            "PayToAll" => {
                let amount = result["amount"].clone().as_int().unwrap();
                let payer_id = self.state.players[player_index].id;
                let players_count = (self.state.players.len()) as u32;

                // ... (payer_id를 제외한 모든 플레이어들과 정부에게 amount씩 더하고, payer에게서는 총액을 빼는 로직) ...
                for player in self.state.players.iter_mut() {
                    if player.id == payer_id {
                        player.money -= amount * players_count as i64;
                    } else {
                        player.money += amount;
                    }
                }
                self.state.government_income += amount;

                self.state.log.push(format!("Paid ${} per each to other players.", amount));

                if self.state.players[player_index].money < 0 {
                    self.prompt_financial_crisis();
                }
            },
            "AllEarn" => {
                let trigger_id = self.state.players[player_index].id;
                let amount_unit = result["amount_unit"].clone().as_int().unwrap();
                self.state.players.iter_mut().for_each(move |player| {
                    let ratio = if player.id == trigger_id {
                        2
                    } else { 1 };
                    player.money += amount_unit * ratio;
                });
                self.state.government_income += amount_unit;
                self.now = GameSituation::EndTurn;
            }
            "PromptLuckTest" => {
                self.now = GameSituation::PendingLuckTestResponse;
            },
            "PromptFinancialCrisis" => {
                let cost = result["cost"].clone().as_int().unwrap();
                let player_mut = &mut self.state.players[player_index];
                player_mut.money -= cost;
                self.prompt_financial_crisis();
            },
            "Educate" => {
                Self::educate(&mut self.state.players[player_index]);
                self.now = GameSituation::EndTurn;
            },
            "MedicalCare" => {
                let free = result["free"].clone().as_bool().unwrap();
                _ = self.medical_care(free);
            },
            "Concert" => {
                let price = result["price"].clone().as_int().unwrap();
                self.state.players[player_index].money -= price;
                self.state.government_income += price / 10;

                if self.state.players[player_index].money < 0 {
                    self.prompt_financial_crisis();
                }
            },
            "GetRandomChanceCard" => {
                self.now = GameSituation::PendingGetRandomChanceCardResponse;
            },
            "PromptTicket" => {
                let kind = result["kind"].clone().into_string().unwrap();
                self.state.pending_ticket += TicketCount::get_one_ticket(kind.as_str());
                if self.state.pending_ticket != TicketCount::zero() {
                    self.now = GameSituation::PendingUseTicketResponse
                }
            }
            _ => { // Log
                let message = result["message"].clone().into_string().unwrap();
                self.state.log.push(message);
            }
        }

        if let GameSituation::InAction = self.now {
            self.now = GameSituation::EndTurn;
        }

        // 최종 위치 업데이트
        self.state.players[player_index].position = new_pos;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn use_ticket(&mut self, to_use: TicketCount, script_action: &str, script_cycle: &str) -> Result<(), String> {
        if let GameSituation::PendingUseTicketResponse = self.now {
            let player_index = self.state.current_turn_idx;
            let position = self.state.players[player_index].position;
            match self.state.board[position as usize].tile_type.as_str() {
                "LuckTest" => {
                    self.now = GameSituation::PendingLuckTestResponse;
                    if to_use.double_lotto > 0 {
                        self.state.players[player_index].tickets_count.double_lotto -= 1;
                        self.luck_test(true);
                    }
                },
                "Jail" => {
                    if to_use.release_from_jail > 0 {
                        self.state.players[player_index].tickets_count.release_from_jail -= 1;
                        self.state.players[player_index].remaining_jail_turns = 0;
                    }
                    self.now = GameSituation::EndTurn;
                },
                "Hospital" => {
                    if to_use.free_hospital > 0 {
                        self.state.players[player_index].tickets_count.free_hospital -= 1;
                    }
                    _ = self.medical_care(to_use.free_hospital > 0);
                    self.now = GameSituation::EndTurn;
                },
                "Property" | "IndustrialComplex" => {
                    let to_use_ticket = if to_use.free_property > 0 {
                        self.state.players[player_index].tickets_count.free_property -= 1;
                        1_i64
                    } else {
                        -1_i64
                    };
                    let result = self.try_run_turn_script(script_action, None, script_cycle, to_use_ticket);
                    if let Err(e) = result {
                        return Err(e);
                    }
                }
                "Tax" => {
                    let to_use_ticket = if to_use.no_tax > 0 {
                        self.state.players[player_index].tickets_count.no_tax -= 1;
                        1_i64
                    } else {
                        -1_i64
                    };
                    let result = self.try_run_turn_script(script_action, None, script_cycle, to_use_ticket);
                    if let Err(e) = result {
                        return Err(e);
                    }
                },
                _ => {
                    return Ok(());
                }
            }
        }
        Ok(())

    }

    #[wasm_bindgen]
    pub fn luck_test(&mut self, init_double_lotto: bool) {
        if let GameSituation::PendingLuckTestResponse =  self.now && self.state.luck_test_cache != 0_i64 {
            let randvar = rand::random_bool(1.0/10.0);
            let result = if !randvar {
                0_i64
            } else if self.state.luck_test_cache < 0 {
                if init_double_lotto {
                    1000000_i64
                } else {
                    500000_i64
                }
            } else {
                self.state.luck_test_cache * 2
            };
            self.state.luck_test_cache = result;
        }
        if self.state.luck_test_cache == 0_i64 {
            self.now = GameSituation::EndTurn;
        }
    }

    fn medical_care(&mut self, free: bool) -> bool {
        let hospital_pos = self.state.board.iter().position(|t| t.tile_type == "Hospital").unwrap();
        let hospital_cost = self.state.board[hospital_pos].amount / 2;

        let player_index = self.state.current_turn_idx;
        let player_mut = &mut self.state.players[player_index];

        self.state.log.push("Sent to Hospital!".into());

        let tmp = self.state.government_income - hospital_cost;

        if !free {
            player_mut.money -= hospital_cost;
        }

        if tmp < 0 {
            self.state.government_income = 0;
            if !free {
                player_mut.money += tmp
            }
        } else {
            self.state.government_income = tmp;
        }

        let crisis = player_mut.money < 0;
        if player_mut.money < 0 {
            self.prompt_financial_crisis();
        }
        return !crisis;
    }

    #[wasm_bindgen]
    pub fn run_turn_script(&mut self, script_action: &str, dices: DicePair, script_cycle: &str) -> Result<(), String> {
        self.try_run_turn_script(script_action,Some(dices),script_cycle,0)
    }

    fn prompt_financial_crisis(&mut self) {
        self.now = GameSituation::PendingFinancialCrisisResponse;
    }

    fn educate(player_mut: &mut Player) {
        player_mut.education_status.educate();
    }

    #[wasm_bindgen]
    pub fn buy(&mut self, pos: u32) {
        let player_index = self.state.current_turn_idx;
        let player_mut = &mut self.state.players[player_index];
        let name = self.state.board[pos as usize].name.clone();
        player_mut.money -= self.building_cost;

        self.state.log.push(format!("Player {} bought '{}'!", player_mut.id, name));
        if let Some((_, v)) = self.state.properties.get_mut(&name) {
            let tmp = min(3u32, *v + 1);
            *v = tmp;
        } else {
            self.state.properties.insert(name, (player_mut.id, 1u32));
        }
        self.now = GameSituation::EndTurn;
    }

    fn trigger_cycle(&mut self, script: &str) -> Result<(), String> {
        let salary = self.salary;
        let government_income = self.state.government_income;
        let player_mut = &mut self.state.players[self.state.current_turn_idx];
        player_mut.cycles += 1;
        let money = player_mut.money;
        let education_status = player_mut.education_status.clone();
        let sum_of_all_taxes = self.state.board.iter().filter_map(|tile| {
            if tile.tile_type == "Infrastructure" {
                Some(tile.amount)
            } else if tile.tile_type == "Hospital" {
                Some(tile.amount / 2)
            } else {
                None
            }
        }).sum::<i64>();

        let mut scope = Scope::new();
        scope.push_constant("salary", salary);
        scope.push("government_income", government_income);
        scope.push_constant("sum_of_all_taxes", sum_of_all_taxes);
        scope.push("money", money);
        scope.push_constant("is_graduated", if let EducationStatus::Graduated = education_status { true } else { false });
        scope.push_constant("has_bonus", player_mut.tickets_count.bonus > 0);

        let result: Map = self.engine.eval_with_scope(&mut scope, script).map_err(|e| e.to_string())?;
        let new_government_income = result["new_government_income"].clone().as_int().unwrap();
        let remaining_salary = result["remaining_salary"].clone().as_int().unwrap();
        let basic_income = result["basic_income"].clone().as_int().unwrap();

        self.state.government_income = new_government_income;
        player_mut.money += remaining_salary;
        self.state.players.iter_mut().for_each(|each_player_mut| {
            each_player_mut.money += basic_income;
        });
        if self.state.players[self.state.current_turn_idx].tickets_count.bonus > 0 {
            self.state.pending_ticket.bonus -= 1;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn borrow_money(&mut self, pid: u32, amount: i64) {
        let found = self.state.players.iter_mut().find(|player| player.id == pid);
        if let Some(player_mut) = found {
            if amount > 0 {
                let loans_acc = if let Some(&(lid, _, _)) = player_mut.remaining_loans.iter().max_by_key(|&&(lid,_,_)| lid) {
                    lid+1
                } else {
                    0u32
                };
                
                player_mut.remaining_loans.push((loans_acc, amount, 4u32));
                player_mut.money += amount;
            }
        }
    }

    #[wasm_bindgen]
    pub fn repay_loan(&mut self, pid: u32, lid: u32, amount: i64) {
        let found = self.state.players.iter_mut().find(|player| player.id == pid);
        if let Some(player_mut) = found {
            if amount > 0 {
                if let Some((_, rem_amount, _)) = player_mut.remaining_loans.iter_mut().find(|(loan_id, rem_amount, _)| *loan_id == lid && *rem_amount > 0) {
                    *rem_amount -= amount;
                    player_mut.money -= amount;
                    player_mut.money -= amount / 10;
                }
                player_mut.remaining_loans.retain(|(_, rem_amount, _)| *rem_amount > 0);
            }
        }
    }

    /// 턴을 종료하고 다음 플레이어로 넘기는 함수
    #[wasm_bindgen]
    pub fn end_turn(&mut self) {
        self.garbage_collect();
        if let GameSituation::EndTurn = self.now {
            let position = self.state.players[self.state.current_turn_idx].position as usize;
            let is_in_jail = self.state.board[position].tile_type == "Jail";
            if !self.state.dice_double || is_in_jail {
                self.state.current_turn_idx = (self.state.current_turn_idx + 1) % self.state.players.len();
                Self::consume_counter(&mut self.state.catastrophe_counter);
                Self::consume_counter(&mut self.state.pandemic_counter);
            }
            self.state.dice_double = false;
            self.state.log.push(format!("--- End of Turn ---"));
            self.before_begin_turn();
        }
    }

    fn before_begin_turn(&mut self) {
        let before_four_cycles = self.state.players.iter().filter_map(|player| if player.cycles < 4 { Some(player.id) } else { None }).collect::<Vec<_>>();
        let current_turn_idx = self.state.current_turn_idx;
        let player = &self.state.players[current_turn_idx];
        let position = player.position;
        let tile = &self.state.board[position as usize];

        if before_four_cycles.is_empty() {
            self.state.log.push(format!("The game has ended."));
            self.now = GameSituation::EndGame;
        } else if tile.tile_type == "Jail" && player.remaining_jail_turns > 0 {
            self.state.log.push(format!("It is now Player {}'s turn.", player.id));
            self.now = GameSituation::PendingTryToJailbreakResponse;
        } else {
            self.state.log.push(format!("It is now Player {}'s turn.", player.id));
            self.now = GameSituation::PendingRollResponse;
        }
    }

    #[wasm_bindgen]
    pub fn try_to_jailbreak_by_dices(&mut self, dices: DicePair) {
        let current_turn_idx = self.state.current_turn_idx;
        let player_mut = &mut self.state.players[current_turn_idx];
        if dices.is_double() {
            player_mut.remaining_jail_turns = 0;
        }
        self.now = GameSituation::EndTurn;
    }

    #[wasm_bindgen]
    pub fn give_up_jailbreak(&mut self) {
        let current_turn_idx = self.state.current_turn_idx;
        let player_mut = &mut self.state.players[current_turn_idx];
        if player_mut.remaining_jail_turns > 0 {
            player_mut.remaining_jail_turns -= 1;
        }
        self.now = GameSituation::EndTurn;
    }

    #[wasm_bindgen]
    pub fn try_to_jailbreak_by_money(&mut self) {
        let current_turn_idx = self.state.current_turn_idx;
        let player_mut = &mut self.state.players[current_turn_idx];
        let amount = self.state.board.iter().find_map(|tile| if tile.tile_type == "Jail" { Some(tile.amount) } else { None }).unwrap();
        if player_mut.money >= amount {
            player_mut.remaining_jail_turns = 0;
            player_mut.money -= amount;
            self.now = GameSituation::EndTurn;
        }
    }

    #[wasm_bindgen]
    pub fn get_random_chance_card(&mut self) {
        let rand_card_entry_idx = rand::random::<u32>() as usize % self.state.chance_cards_inventory.len();
        let card_id = self.state.chance_cards_inventory.keys().collect::<Vec<_>>()[rand_card_entry_idx].clone();
        
        self.pending_chance_card_id = Some(card_id);
        self.now = GameSituation::PendingCheckChanceCardResponse;
    }

    fn property_swap(&mut self, to_give: &String, to_get: &String) {
        let pair = (self.state.properties[to_give].0.clone(), self.state.properties[to_get].0.clone());
        self.state.properties.iter_mut().for_each(|(name, (owner_id,_ ))| {
            if *name == *to_give {
                *owner_id = pair.1;
            } else if *name == *to_get {
                *owner_id = pair.0;
            }
        });
    }

    #[wasm_bindgen]
    pub fn check_chance_card(&mut self, script_chance_action: &str, script_cycle: &str, payload_json: Option<String>) -> Result<(), String> {
        if let Some(cid) = &self.pending_chance_card_id {
            
            let current_turn_idx = self.state.current_turn_idx;
            let player_mut = &mut self.state.players[current_turn_idx];
            let player_money = player_mut.money.clone();

            let mut scope = Scope::new();
            scope.push("card_id", cid.clone());
            let payload = if let Some(s) = payload_json {
                let json_str = s.as_str();
                self.engine.parse_json(r#json_str, true).map_err(|e| e.to_string())?
            } else {
                self.engine.parse_json(r#"{}"#, true).map_err(|e| e.to_string())?
            };
            scope.push("payload", payload);

            let (my_properties, others_properties) = Self::get_owned_properties(&self.state.properties,player_mut.id);
            let my_houses_countsum = my_properties.iter().filter_map(|(name, count)| {
                let tile_type = self.state.board.iter().find_map(|tile| {
                    if tile.name == *name {
                        Some(tile.tile_type.clone())
                    } else {
                        None
                    }
                });
                if let Some(tt) = tile_type && tt == "Property" {
                    Some(*count as i64)
                } else {
                    None
                }
            }).sum::<i64>();
            scope.push("my_properties", my_properties);
            scope.push("my_houses_countsum", my_houses_countsum);
            scope.push("others_properties", others_properties);
            scope.push("player_money", player_money);

            let result: Map = self.engine.eval_with_scope(&mut scope, script_chance_action).map_err(|e| e.to_string())?;
            let action_type = result["type"].clone().into_string().unwrap();

            match action_type.as_str() {
                "Earn" => {
                    let amount = result["amount"].clone().as_int().unwrap();
                    player_mut.money += amount;
                    self.now = GameSituation::EndTurn;
                },
                "Earthquake" => {
                    let tmp = self.state.properties.iter().filter_map(|(name, (owner_id, owned_amount))| {
                        if *owner_id == player_mut.id {
                            if *owned_amount > 1 {
                                Some((name.clone(), (*owner_id, (*owned_amount) - 1)))
                            } else {
                                None
                            }
                        } else {
                            Some((name.clone(), (*owner_id, *owned_amount)))
                        }
                    }).collect::<HashMap<_, _>>();
                    self.state.properties = tmp;
                    self.now = GameSituation::EndTurn;
                },
                "GoToJail" => {
                    let jail_pos = self.state.board.iter().position(|t| t.tile_type == "Jail").unwrap();
                    player_mut.position = jail_pos as u32;
                    self.state.log.push("Sent to Jail!".into());
                    self.now = GameSituation::EndTurn;
                },
                "GoToHospital" => {
                    let hospital_pos = self.state.board.iter().position(|t| t.tile_type == "Hospital").unwrap();

                    player_mut.position = hospital_pos as u32;
                    if self.state.players[self.state.current_turn_idx].tickets_count.free_hospital > 0 {
                        self.now = GameSituation::PendingUseTicketResponse;
                    } else {
                        let crisis = self.medical_care(false);
                        if !crisis {
                            self.now = GameSituation::EndGame;
                        }
                    }
                },
                "GoToUniversity" => {
                    let univ_pos = self.state.board.iter().position(|t| t.tile_type == "University").unwrap();
                    player_mut.position = univ_pos as u32;
                    self.state.log.push("Sent to University!".into());
                    Self::educate(player_mut);
                    self.now = GameSituation::EndTurn;
                },
                "GetTicket" => {
                    let kind = result["kind"].clone().into_string().unwrap();
                    self.state.players[self.state.current_turn_idx].tickets_count += TicketCount::get_one_ticket(kind.as_str());
                    self.now = GameSituation::EndTurn;
                },
                "TwistOfFate" => {
                    let dice_a = result["dice_a"].clone().as_int().unwrap() as usize;
                    let dice_b = result["dice_b"].clone().as_int().unwrap() as usize;
                    let players_count = self.state.players.len();
                    let current_turn_idx = self.state.current_turn_idx;
                    let target_turn_idx = (current_turn_idx + dice_a + dice_b) % players_count;
                    let swap_result = self.swap_all_properties(current_turn_idx, target_turn_idx);
                    if swap_result {
                        self.now = GameSituation::EndTurn;
                    }
                },
                "PayTo" => {
                    let player_index = self.state.current_turn_idx;
                    let government_amount = if let Ok(amount) = result["gov_amount"].clone().as_int() {
                        Some(amount)
                    } else { None };
                    let market_amount = if let Ok(amount) = result["market_amount"].clone().as_int() {
                        Some(amount)
                    } else { None };
                    let to_player = if let Ok(amount) = result["player_amount"].clone().as_int() && let Ok(pid) = result["to_player_id"].clone().as_int() {
                        Some((amount, pid as u32))
                    } else { None };
                    let payer_id = self.state.players[player_index].id;

                    let message = result["message"].clone().into_string().unwrap();
                    self.state.log.push(message);

                    if let Some(amount) = government_amount {
                        self.state.government_income += amount;
                        self.state.players[player_index].money -= amount;
                        self.state.log.push(format!("\tPlayer {} Paid ${} to the government.", payer_id, amount));
                    }

                    if let Some(amount) = market_amount {
                        self.state.players[player_index].money -= amount;
                        self.state.log.push(format!("\tPlayer {} Paid ${} to the market.", payer_id, amount));
                    }

                    if let Some((amount, pid)) = to_player {
                        let to_player = self.state.players.iter_mut().find(|player| player.id == pid);
                        if let Some(to_player_mut) = to_player {
                            to_player_mut.money += amount;
                            self.state.players[player_index].money -= amount;
                            self.state.log.push(format!("\tPlayer {} Paid ${} to Player {}.", payer_id, amount, pid));
                        }
                    }

                    if self.state.players[player_index].money < 0 {
                        self.prompt_financial_crisis();
                    } else {
                        self.now = GameSituation::EndTurn;
                    }
                },
                "WarpToPosition" => {
                    let dest = result["position"].clone().as_int().unwrap() as u32;
                    player_mut.position = dest;
                    self.state.log.push(format!("Warped to {}!", self.state.board[dest as usize].name));
                    self.now = GameSituation::EndTurn;
                },
                "TravelToPosition" => {
                    let dest = result["position"].clone().as_int().unwrap() as u32;
                    let old_pos = player_mut.position.clone();
                    player_mut.position = dest;
                    self.state.log.push(format!("Traveled to {}!", self.state.board[dest as usize].name));
                    if old_pos >= dest {
                        self.trigger_cycle(script_cycle)?;
                    }
                    self.now = GameSituation::EndTurn;
                },
                "DestructOnePerEach" => {
                    let raw_targets = result["targets"].clone().into_array().unwrap();
                    let processed_targets = raw_targets.iter().filter_map(|item| {
                        if item.type_name() == "string" {
                            Some(item.clone().into_string().unwrap())
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>();
                    let targets = processed_targets.iter().map(|s| s.as_str()).collect::<Vec<_>>();
                    self.state.properties.iter_mut().for_each(|(name, (_, owned_amount))| {
                        let name_as_str = name.as_str();
                        if targets.contains(&name_as_str) {
                            if *owned_amount > 0 {
                                *owned_amount -= 1;
                            }
                        }
                    });
                    self.now = GameSituation::EndTurn;
                },
                "Pandemic" => {
                    self.state.pandemic_counter += self.state.players.len() + 1;
                    self.now = GameSituation::EndTurn;
                },
                "FreeConstruction" => {
                    let target = result["target"].clone().into_string().unwrap();
                    let max_buildings = if let Some(&max_buildings) = self.state.consts.get("MAX_BUILDINGS") && max_buildings > 0 {
                        max_buildings
                    } else {
                        1
                    };
                    if let Some((owner_id, owned_amount)) = self.state.properties.get_mut(&target) && *owner_id == player_mut.id && *owned_amount < max_buildings {
                        *owned_amount += 1;
                        self.now = GameSituation::EndTurn;
                    }
                },
                "Catastrophe" => {
                    self.state.catastrophe_counter += self.state.players.len() + 1;
                    self.now = GameSituation::EndTurn;
                },
                "NOP" => {
                    self.now = GameSituation::EndTurn;
                },
                "GoToPayElectricityFee" => {
                    let using_ticket = result["using_ticket"].clone().as_bool().unwrap();
                    let (elec_pos, elec_tile) = self.state.board.iter().enumerate().find(|&(_, tile)| tile.name.as_str() == "Electricity").unwrap();
                    player_mut.position = elec_pos as u32;
                    self.state.log.push("Sent to Electricity!".into());
                    
                    if using_ticket && player_mut.tickets_count.no_tax > 0 {
                        player_mut.tickets_count.no_tax -= 1;
                    } else {
                        player_mut.money -= elec_tile.amount;
                    }

                    if player_mut.money < 0 {
                        self.prompt_financial_crisis();
                    } else {
                        self.now = GameSituation::EndTurn;
                    }
                },
                "GraduateNow" => {
                    player_mut.education_status = EducationStatus::Graduated;
                    self.now = GameSituation::EndTurn;
                },
                "PropertySwap" => {
                    let to_get = result["to_get"].clone().into_string().unwrap();
                    let to_give = result["to_give"].clone().into_string().unwrap();
                    self.property_swap(&to_give, &to_get);
                    self.now = GameSituation::EndTurn;
                }
                // ...
                _ => {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn consume_counter(counter: &mut usize) {
        if *counter > 0 {
            *counter -= 1;
        }
    }

    #[inline(always)]
    fn garbage_collect(&mut self) {
        _ = self.state.properties.extract_if(|_, (_, owned_amount)| {
            *owned_amount == 0u32
        });
    }

    fn swap_all_properties(&mut self, a_turn_idx: usize, b_turn_idx: usize) -> bool {
        if a_turn_idx == b_turn_idx {
            false
        } else {
            let (a_id, b_id) = (self.state.players[a_turn_idx].id, self.state.players[b_turn_idx].id);
            let cloned = self.state.properties.clone();
            let of_a = cloned.iter().filter_map(|(name, (owner_id, owned_amount))| {
                if *owner_id == a_id && *owned_amount > 0 {
                    Some(name.as_str())
                } else { None }
            }).collect::<Vec<_>>();
            let of_b = cloned.iter().filter_map(|(name, (owner_id, owned_amount))| {
                if *owner_id == b_id && *owned_amount > 0 {
                    Some(name.as_str())
                } else { None }
            }).collect::<Vec<_>>();
            self.state.properties.iter_mut().for_each(|(name, (owner_id, _))| {
                if of_a.contains(&name.as_str()) {
                    *owner_id = b_id;
                } else if of_b.contains(&name.as_str()) {
                    *owner_id = a_id;
                }
            });
            true
        }
    }

    #[wasm_bindgen]
    pub fn get_state_as_json(&self) -> String {
        serde_json::to_string(&self.state).unwrap()
    }
}