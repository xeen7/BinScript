use crate::core::vtable::DATE_VTABLE;
use crate::core::alloc::__bs_alloc;
use crate::types::string_utils::get_c_string_from_tagged;
use crate::objects::dynamic_props::set_dynamic_property;

fn parse_date_string(s: &str) -> f64 {
    if let Ok(ms) = s.parse::<f64>() {
        return ms;
    }
    let parts: Vec<&str> = s.split(|c| c == '-' || c == '/').collect();
    if parts.len() >= 3 {
        let t_parts: Vec<&str> = parts[2].split('T').collect();
        let day_str = t_parts.first().copied().unwrap_or("");
        let time_str = t_parts.get(1).copied().unwrap_or("");

        if let (Ok(y), Ok(m), Ok(d)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>(), day_str.parse::<u32>()) {
            let y_from_epoch = y - 1970;
            let leap_years = if y_from_epoch >= 0 {
                (y_from_epoch + 1) / 4
            } else {
                (y_from_epoch - 2) / 4
            };
            let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut days = y_from_epoch * 365 + leap_years;
            for i in 0..(m.saturating_sub(1) as usize) {
                if i < 12 {
                    days += days_in_months[i];
                }
            }
            if m > 2 && y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                days += 1;
            }
            days += (d as i32) - 1;

            let mut ms = (days as f64) * 86400.0 * 1000.0;

            if !time_str.is_empty() {
                let hms_str = time_str.split(|c: char| c == 'Z' || c == '+' || c == '-').next().unwrap_or("");
                let hms_parts: Vec<&str> = hms_str.split(':').collect();
                let hours = hms_parts.get(0).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let minutes = hms_parts.get(1).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                let seconds = hms_parts.get(2).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0);

                ms += hours * 3600.0 * 1000.0;
                ms += minutes * 60.0 * 1000.0;
                ms += seconds * 1000.0;
            }
            return ms;
        }
    }
    0.0
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Date_new(val: u64) -> u64 {
    if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        __bs_Date_new_0()
    } else {
        __bs_Date_new_1(val)
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Date_new_0() -> u64 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    let ms = since_the_epoch.as_millis() as f64;
    let obj = __bs_alloc(&DATE_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), crate::circ::box_number(ms));
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Date_new_1(val: u64) -> u64 {
    let ms = if (val & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 {
        f64::NAN
    } else if (val & 0xFFFF_0000_0000_0000) == 0xFFF2_0000_0000_0000 {
        0.0
    } else {
        let tag = val & 0xFFFF_0000_0000_0000;
        if tag == 0xFFF7_0000_0000_0000 {
            let s = get_c_string_from_tagged(val);
            parse_date_string(s)
        } else {
            f64::from_bits(val)
        }
    };
    let obj = __bs_alloc(&DATE_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), crate::circ::box_number(ms));
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_Date_new_n(
    y_tagged: u64,
    m_tagged: u64,
    d_tagged: u64,
    h_tagged: u64,
    min_tagged: u64,
    s_tagged: u64,
    ms_tagged: u64,
) -> u64 {
    let y = if (y_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 1970 } else { f64::from_bits(y_tagged) as i32 };
    let m = if (m_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(m_tagged) as u32 };
    let d = if (d_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 1 } else { f64::from_bits(d_tagged) as u32 };
    let h = if (h_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(h_tagged) as u32 };
    let min = if (min_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(min_tagged) as u32 };
    let s = if (s_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0 } else { f64::from_bits(s_tagged) as u32 };
    let ms_val = if (ms_tagged & 0xFFFF_0000_0000_0000) == 0xFFF1_0000_0000_0000 { 0.0 } else { f64::from_bits(ms_tagged) };

    let adjusted_y = if y >= 0 && y <= 99 { 1900 + y } else { y };

    let y_from_epoch = adjusted_y - 1970;
    let leap_years = if y_from_epoch >= 0 {
        (y_from_epoch + 1) / 4
    } else {
        (y_from_epoch - 2) / 4
    };
    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days = y_from_epoch * 365 + leap_years;
    for i in 0..(m as usize) {
        if i < 12 {
            days += days_in_months[i];
        }
    }
    let is_leap = adjusted_y % 4 == 0 && (adjusted_y % 100 != 0 || adjusted_y % 400 == 0);
    if m > 1 && is_leap {
        days += 1;
    }
    days += (d as i32) - 1;

    let mut epoch_ms = (days as f64) * 86400.0 * 1000.0;
    epoch_ms += (h as f64) * 3600.0 * 1000.0;
    epoch_ms += (min as f64) * 60.0 * 1000.0;
    epoch_ms += (s as f64) * 1000.0;
    epoch_ms += ms_val;

    let obj = __bs_alloc(&DATE_VTABLE, 16);
    let payload = obj & 0x0000_FFFF_FFFF_FFFF;
    set_dynamic_property(payload as *mut u8, "[[PrimitiveValue]]".to_string(), crate::circ::box_number(epoch_ms));
    obj
}

#[no_mangle]
pub unsafe extern "C-unwind" fn __bs_date_now() -> u64 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
    crate::circ::box_number(since_the_epoch.as_millis() as f64)
}