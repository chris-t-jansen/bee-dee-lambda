pub fn month_num_to_str(month_num: u8) -> String {
    match month_num {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => panic!("Month number wasn't in the range of 1-12!")
    }.to_string()
}

pub fn day_num_to_ordinal(day_num: u8) -> String {
    if 20 > day_num && day_num > 10 {
        return format!("{}th", day_num);
    } else {
        let ones_place = day_num % 10;
        return match ones_place {
            1 => format!("{}st", day_num),
            2 => format!("{}nd", day_num),
            3 => format!("{}rd", day_num),
            _ => format!("{}th", day_num),
        };
    }
}