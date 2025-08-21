// Rust에서 현재 타일 정보(tile)와 소유 여부(is_owned)를 받음
switch tile.type {
    "Property" => {
        if is_owned {
            #{ type: "Log", message: `Landed on owned property '${tile.name}'.` };
        } else {
            #{ type: "PromptBuy", tile_name: tile.name, price: tile.price };
        }
    },
    "Tax" => #{ type: "PayTax", amount: tile.amount },
    "GoToJail" => #{ type: "GoToJail" },
    _ => #{ type: "Log", message: `Landed on '${tile.name}'.` }
}