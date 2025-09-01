let pandemic_flag = during_pandemic();
// Rust에서 현재 타일 정보(tile)와 소유 여부(is_owned)를 받음
switch tile.type {
    "Property" => {
        let ticket_flag = tickets.free_property > 0 && to_use_ticket > 0;
        if tickets.free_property > 0 && to_use_ticket == 0 {
            #{ type: "PromptTicket", kind: "FreeProperty" };
        }
        let catastrophe_flag_flag = tile.is_megacity && is_in_catastrophe();
        let free_flag = catastrophe_flag || pandemic_flag;
        if player.money < tile.price && !free_flag {
            if ticket_flag {
                #{ type: "UseTicket", kind: "FreeProperty" };
            } else if is_owned && owner_id != player.id {
                #{ type: "PromptFinancialCrisis", cost: tile.price + (building_cost * owned_amount) };
            } else {
                #{ type: "PromptFinancialCrisis", cost: tile.price };
            }
        } else if is_owned && owner_id != player.id {
            if free_flag {
                #{ type: "Log", message: Landed on property '${tile.name}' owned by Player ${owner_id}.` };
            } else if ticket_flag {
                #{ type: "UseTicket", kind: "FreeProperty" };
            } else {
                #{ type: "PayTo", gov_amount: tile.price, to_player_id: owner_id, player_amount: building_cost * owned_amount, message: Landed on property '${tile.name}' owned by Player ${owner_id}.` };
            }
        } else if is_owned && owner_id == player.id && owned_amount >= MAX_BUILDINGS {
            if free_flag {
                #{ type: "Log", message: `Landed on property '${tile.name}' without any space left.` };
            } else if ticket_flag {
                #{ type: "UseTicket", kind: "FreeProperty" };
            } else {
                #{ type: "PayTo", gov_amount: tile.price, message: `Landed on property '${tile.name}' without any space left.` };
            }
        } else  {
            #{ type: "PromptBuy", free_flag: free_flag, ticket_flag: ticket_flag, tile_name: tile.name, price: tile.price };
        }
    },
    
    "Tax" => {
        if tickets.no_tax > 0 && to_use_ticket == 0 {
            #{ type: "PromptTicket", kind: "NoTax" };
        } else if tickets.no_tax > 0 && to_use_ticket > 0 {
            #{ type: "UseTicket", kind: "NoTax" };
        } else {
            #{ type: "PayTax", free_flag: pandemic_flag, amount: tile.amount }
        }
    },
    
    "GoToJail" => #{ type: "GoToJail" },

    "PublicTransport" => {
        // 다음 대중교통 타일의 위치를 Rust에 물어봄
        let next_pos = rust_api_find_next_tile_of_type(player.position, "PublicTransport");
        // 'Warp' 액션을 반환하여 플레이어를 즉시 이동시킴
        #{ type: "WarpToPosition", position: next_pos };
    },

    "IndustrialComplex" => {
        let ticket_flag = tickets.free_property > 0 && to_use_ticket > 0;
        if tickets.free_property > 0 && to_use_ticket == 0 {
            #{ type: "PromptTicket", kind: "FreeProperty" };
        }
        let catastrophe_flag_flag = tile.is_megacity && is_in_catastrophe();
        let free_flag = catastrophe_flag || pandemic_flag;
        if is_owned {
            if owner_id != player.id {
                let num_players = get_player_count();
                if num_players > 1 {
                    if free_flag {
                        let to_earn_unit = tile.price / (num_players+2);
                        #{ type: "AllEarn", amount_unit: to_earn_unit, message: "비상상황이라 지원금으로 메꿨습니다." };
                    } else {
                        let fee = tile.price / num_players;
                        #{ type: "PayToAll", amount: fee };
                    }
                } else {
                    #{ type: "Log", message: "혼자라 해당사항 없습니다." };
                }
            } else {
                if num_players > 1 {
                    let to_earn_unit = tile.price / (num_players+2);
                    #{ type: "AllEarn", amount_unit: to_earn_unit, message: "산업의 혜택을 분배했습니다." };
                } else {
                    #{ type: "Log", message: "혼자라 해당사항 없습니다." };
                }
            }
        } else {
            #{ type: "PromptBuy", tile_name: tile.name, price: tile.price };
        }
    },

    "Park" => {
        // 아무 효과 없음
        #{ type: "Log", message: "공원에서 잠시 휴식합니다." };
    },
    
    "Concert" => {
        #{ type: "Concert", price: tile.price, message: "" };
    },

    "University" => { // 대학교
        #{ type: "Educate" };
    },

    "Infrastructure" => { // 인프라
        if pandemic_flag {
            #{ type: "Log", message: "판데믹으로 인해 요금을 한시적으로 면제받았습니다." };
        } else {
            #{ type: "PayTo", gov_amount: tile.amount, message: `Landed on infrastructure '${tile.name}':` };
        }
    },

    "Hospital" => { // 병원
        let ticket_flag = tickets.hospital > 0 && to_use_ticket > 0;
        if tickets.free_hospital > 0 && to_use_ticket == 0 {
            #{ type: "PromptTicket", kind: "FreeHospital" };
        } else if ticket_flag {
            #{ type: "MedicalCare". free: true }
        } else {
            #{ type: "MedicalCare", free: false }
        }
    },

    "ChanceCard" => { // 변화카드 뽑기
        #{ type: "GetRandomChanceCard" }
    }

    _ => { // 기타 모든 경우
        #{ type: "Log", message: `Landed on '${tile.name}'.` }
    }
}