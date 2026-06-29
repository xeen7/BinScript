use crate::dynamic_call::helpers::{TAG_MASK, TAG_OBJECT, PAYLOAD_MASK};

pub struct DateComponents {
    pub year: i32,
    pub month: u32,
    pub date: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

pub fn ms_to_components(ms: f64) -> DateComponents {
    let seconds = (ms / 1000.0).floor() as i64;
    let mut seconds_of_day = seconds % 86400;
    if seconds_of_day < 0 {
        seconds_of_day += 86400;
    }
    let hours = (seconds_of_day / 3600) as u32;
    let minutes = ((seconds_of_day % 3600) / 60) as u32;
    let seconds_ret = (seconds_of_day % 60) as u32;

    let mut days = seconds / 86400;
    if seconds % 86400 < 0 && seconds % 86400 != 0 {
        days -= 1;
    }

    let mut year = 1970i32;
    if days >= 0 {
        loop {
            let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let days_in_year: i64 = if leap { 366 } else { 365 };
            if days >= days_in_year {
                days -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }
    } else {
        loop {
            year -= 1;
            let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let days_in_year: i64 = if leap { 366 } else { 365 };
            days += days_in_year;
            if days >= 0 {
                break;
            }
        }
    }

    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let mut days_in_months = [31i64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if leap {
        days_in_months[1] = 29;
    }

    let mut month = 0usize;
    while days >= days_in_months[month] {
        days -= days_in_months[month];
        month += 1;
    }

    DateComponents {
        year,
        month: month as u32,
        date: (days + 1) as u32,
        hours,
        minutes,
        seconds: seconds_ret,
    }
}

pub fn date_to_string(ms: f64) -> String {
    let comps = ms_to_components(ms);
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let seconds = (ms / 1000.0).floor() as i64;
    let mut days = seconds / 86400;
    if seconds % 86400 < 0 && seconds % 86400 != 0 {
        days -= 1;
    }
    let mut wday = (days + 4) % 7;
    if wday < 0 {
        wday += 7;
    }
    let weekdays = ["Sun","Mon","Tue","Wed","Thu","Fri","Sat"];
    format!(
        "{} {} {:02} {} {:02}:{:02}:{:02} GMT",
        weekdays[wday as usize],
        months[comps.month as usize],
        comps.date,
        comps.year,
        comps.hours,
        comps.minutes,
        comps.seconds
    )
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_getTime(recv: u64) -> u64 {
    let tag = recv & TAG_MASK;
    if tag == TAG_OBJECT {
        let payload = recv & PAYLOAD_MASK;
        let obj_ptr = payload as *mut u8;
        if let Some(prim) = crate::objects::dynamic_props::get_dynamic_property(obj_ptr, "[[PrimitiveValue]]") {
            return prim;
        }
    }
    crate::circ::box_number(0.0)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_getFullYear(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    crate::circ::box_number(ms_to_components(ms).year as f64)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_getMonth(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    crate::circ::box_number(ms_to_components(ms).month as f64)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_getDate(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    crate::circ::box_number(ms_to_components(ms).date as f64)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_getHours(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    crate::circ::box_number(ms_to_components(ms).hours as f64)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_getMinutes(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    crate::circ::box_number(ms_to_components(ms).minutes as f64)
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_getSeconds(recv: u64) -> u64 {
    let ms = f64::from_bits(__bs_date_getTime(recv));
    crate::circ::box_number(ms_to_components(ms).seconds as f64)
}
