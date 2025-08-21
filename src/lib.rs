use wasm_bindgen::prelude::*;
use rhai::{Engine, Scope, Map};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

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
}

#[derive(Serialize, Clone, Debug)]
pub struct Player {
    pub id: u32,
    pub position: u32,
    pub money: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct GameState {
    board: Vec<Tile>,
    players: Vec<Player>,
    properties: HashMap<String, u32>,
    log: Vec<String>,
    current_turn_idx: usize,
}

#[wasm_bindgen]
pub struct GameEngine {
    engine: Engine,
    state: GameState,
}

#[wasm_bindgen]
impl GameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(board_json: &str, players_count: usize) -> Result<GameEngine, String> {
        let board: Vec<Tile> = serde_json::from_str(board_json).map_err(|e| e.to_string())?;
        let state = GameState {
            board,
            players: (0..players_count).map(|i| Player { id: (i+1) as u32, position: 0, money: 1500 }).collect(),
            properties: HashMap::new(),
            log: vec!["Game started!".into()],
            current_turn_idx: 0
        };
        let mut engine = Engine::new();

        // Rhai가 Rust 객체를 사용할 수 있도록 등록
        engine.register_type_with_name::<Tile>("Tile");
        engine.register_get("name", |t: &mut Tile| t.name.clone());
        engine.register_get("type", |t: &mut Tile| t.tile_type.clone());
        engine.register_get("price", |t: &mut Tile| t.price);
        engine.register_get("amount", |t: &mut Tile| t.amount);

        Ok(Self { engine, state })
    }

    pub fn run_turn_script(&mut self, script: &str, dice_roll: i64) -> Result<(), String> {
        let mut scope = Scope::new();
        let player_index = self.state.current_turn_idx;
        let player = self.state.players[player_index].clone();
        
        // scope에 현재 플레이어의 id를 넘김
        scope.push("player_id", player.id); 

        self.state.log.push(format!("Player {} rolled a {}", player.id, dice_roll));

        // 이동 후 위치 계산
        let new_pos = (player.position + dice_roll as u32) % self.state.board.len() as u32;
        let tile = self.state.board[new_pos as usize].clone();
        let is_owned = self.state.properties.contains_key(&tile.name);

        scope.push("tile", tile);
        scope.push("is_owned", is_owned);

        let result: Map = self.engine.eval_with_scope(&mut scope, script).map_err(|e| e.to_string())?;

        let action_type = result["type"].clone().into_string().unwrap();
        match action_type.as_str() {
            "PromptBuy" => {
                let name = result["tile_name"].clone().into_string().unwrap();
                let price = result["price"].clone().as_int().unwrap();
                self.state.log.push(format!("Landed on unowned '{}'. Buy for ${}?", name, price));
                // 구매 로직
                let player_mut = &mut self.state.players[player_index];
                if player_mut.money >= price {
                    player_mut.money -= price;
                    self.state.properties.insert(name.clone(), player_mut.id);
                    self.state.log.push(format!("Player 1 bought '{}'!", name));
                } else {
                    self.state.log.push("Not enough money to buy.".into());
                }
            },
            "PayTax" => {
                let amount = result["amount"].clone().as_int().unwrap();
                self.state.players[player_index].money -= amount;
                self.state.log.push(format!("Paid ${} in taxes.", amount));
            },
            "GoToJail" => {
                let jail_pos = self.state.board.iter().position(|t| t.tile_type == "Jail").unwrap_or(10);
                self.state.players[player_index].position = jail_pos as u32;
                self.state.log.push("Sent to Jail!".into());
                return Ok(()); // 이동 로직을 건너뛰기 위해 여기서 종료
            },
            _ => { // Log
                let message = result["message"].clone().into_string().unwrap();
                self.state.log.push(message);
            }
        }
        
        // 최종 위치 업데이트
        self.state.players[player_index].position = new_pos;
        Ok(())
    }

    /// 턴을 종료하고 다음 플레이어로 넘기는 함수
    #[wasm_bindgen]
    pub fn end_turn(&mut self) {
        self.state.current_turn_idx = (self.state.current_turn_idx + 1) % self.state.players.len();
        let next_player_id = self.state.players[self.state.current_turn_idx].id;
        self.state.log.push(format!("--- End of Turn ---"));
        self.state.log.push(format!("It is now Player {}'s turn.", next_player_id));
    }

    pub fn get_state_as_json(&self) -> String {
        serde_json::to_string(&self.state).unwrap()
    }
}