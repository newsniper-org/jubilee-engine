switch chance_type {
    "NewBorn" => {
        #{ type: "Earn", amount: 1000000 };
    },
    "Earthquake" => {
        #{ type: "Earthquake" };
    },
    "TaxEvasion" => {
        #{ type: "GoToJail" };
    },
    "Sick" => {
        #{ type: "GoToHospital" };
    },
    "MedicalCostSupport" => {
        #{ type: "GetTicket", kind: "FreeHospital" };
    },
    "DrugBusted" => {
        #{ type: "GoToJail" };
    },
    "NursingParents" => {
        #{ type: "GoToHospital" };
    },
    "ThePrinceAndThePauper" => {
        #{ type: "TwistOfFate", dice_a: payload["dice_a"], dice_b: payload["dice_b"] };
    },
    "Inheritance" => {
        #{ type: "Earn", amount: 1000000 };
    },
    "Donation" => {
        #{ type: "PayTo", market_amount: 1000000 };
    },
    "Maintenance" => {
        #{ type: "PayTo", market_amount: (my_houses_countsum * 100000) };
    },
    "HealthyDiet" => {
        #{ type: "GetTicket", kind: "FreeHospital" };
    },
    "AccountHijacked" => {
        #{ type: "PayTo", market_amount: 1000000 };
    },
    "Typhoon" => {
        #{ type: "DestructOnePerEach", targets: get_coastal_cities() }
    },
    "PandemicOutbreak" => {
        #{ type: "Pandemic" };
    },
    "ImageMaking" => {
        #{ type: "PayTo", market_amount: 1000000 };
    },
    "GreenNewDeal" => {
        #{ type: "FreeConstruction", target: payload["target"] };
    },
    "VoicePhishing" => {
        #{ type: "PayTo", market_amount: 1000000 };
    },
    "Scholarship" => {
        #{ type: "GoToUniversity" };
    },
    "FreePass" => {
        #{ type: "GetTicket", kind: "FreeProperty"};
    },
    "CatastropheAlert" => {
        #{ type: "Catastrophe" };
    },
    "BONUS" => {
        #{ type: "GetTicket", kind: "Bonus"};
    },
    "DoubleLottoTicket" => {
        #{ type: "GetTicket", kind: "DoubleLotto"};
    },
    "InsiderTradingBusted" => {
        #{ type: "GoToJail" };
    },
    "TrafficJam" => {
        #{ type: "DestructOnePerEach", targets: [payload["target"]] };
    },
    "CharterBus" => {
        #{ type: "TravelToPosition", position: payload["destination"] };
    },
    "TrafficAccident" => {
        #{ type: "PayTo", market_amount: 500000 };
    },
    "NoTaxTicket" => {
        #{ type: "GetTicket", kind: "NoTax"};
    },
    "OverusingElectricity" => {
        #{ type: "GoToPayElectricityFee", using_ticket: payload["using_ticket"] };
    },
    "Lawyer" => {
        #{ type: "GetTicket", kind: "ReleaseFromJail"};
    },
    "Skyrocket" => {
        #{ type: "Earn", amount: round100000(player_money / 2) };
    },
    "Plummet" => {
        #{ type: "PayTo", market_amount: round100000(player_money / 2) };
    },
    "StudyingHard" => {
        #{ type: "GraduateNow" };
    },
    "LowBirth" => {
        #{ type: "DestructOnePerEach", targets: payload["targets"] };
    },
    "Exchange" => {
        #{ type: "PropertySwap", to_give: payload["to_give"], to_get: payload["to_get"] };
    }
}