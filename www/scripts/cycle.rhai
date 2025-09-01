let new_government_income = sum_of_all_taxes;
let basic_income = government_income / get_player_count();
let graduation_factor = if is_graduated { 2 } else { 1 };

#{
    remaining_salary: (salary * graduation_factor) - new_government_income,
    new_government_income: new_government_income,
    basic_income: basic_income
};
