// use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
// use leptos::leptos_dom::helpers::location;
use leptos::logging::error;
use leptos::{document, window};
use wasm_bindgen::JsCast;
use wasm_cookies::SameSite;
use web_sys::HtmlDocument;

// pub fn fmt_date_to_picker(date: &NaiveDate) -> String {
//     format!("{}-{:02}-{:02}", date.year(), date.month(), date.day(),)
// }

// pub fn fmt_dt_to_picker(dt: &DateTime<Utc>, date_only: bool) -> String {
//     if date_only {
//         format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day(),)
//     } else {
//         format!(
//             "{}-{:02}-{:02}T{:02}:{:02}",
//             dt.year(),
//             dt.month(),
//             dt.day(),
//             dt.hour(),
//             dt.minute()
//         )
//     }
// }

// pub fn fmt_dt_to_picker_time(dt: &DateTime<Utc>) -> String {
//     format!("{:02}:{:02}", dt.hour(), dt.minute())
// }

// pub fn pretty_date(date: &NaiveDate) -> String {
//     format!("{:02}.{:02}.{}", date.day(), date.month(), date.year())
// }

// pub fn pretty_datetime(dt: &DateTime<Utc>) -> String {
//     format!(
//         "{:02}.{:02}.{} {}:{:02}",
//         dt.day(),
//         dt.month(),
//         dt.year(),
//         dt.hour(),
//         dt.minute()
//     )
// }

// #[cfg(not(target_arch = "wasm32"))]
// pub fn fmt_picker_to_dt(
//     picker_date: &str,
//     picker_time: &str,
// ) -> Result<DateTime<Utc>, error::ApiError> {
//     use error::{ApiError, ApiErrorType};
//
//     let err = || ApiError::new(ApiErrorType::BadRequest, "cannot parse picker date string");
//     tracing::info!("picker_date {} picker_time {}", picker_date, picker_time);
//     let (year, rest) = picker_date.split_once('-').ok_or(err())?;
//     tracing::info!("year {} rest {}", year, rest);
//     let (month, day) = rest.split_once('-').ok_or(err())?;
//     tracing::info!("month {} day {}", month, day);
//     let (hour, rest) = picker_time.split_once(':').ok_or(err())?;
//     tracing::info!("hour {} rest {}", hour, rest);
//     let (minute, second) = if rest.contains(':') {
//         // in this case, we had hh:mm:ss and not hh:mm
//         rest.split_once(':').unwrap()
//     } else {
//         (rest, "0")
//     };
//
//     let err = || {
//         ApiError::new(
//             ApiErrorType::BadRequest,
//             "cannot create date from parsed value",
//         )
//     };
//     let year = year.parse::<i32>()?;
//     let month = month.parse::<u32>()?;
//     let day = day.parse::<u32>()?;
//     let hour = hour.parse::<u32>()?;
//     let minute = minute.parse::<u32>()?;
//     let second = second.parse::<u32>()?;
//
//     let dt = Utc::now()
//         .with_year(year)
//         .ok_or(err())?
//         .with_month(month)
//         .ok_or(err())?
//         .with_day(day)
//         .ok_or(err())?
//         .with_hour(hour)
//         .ok_or(err())?
//         .with_minute(minute)
//         .ok_or(err())?
//         .with_second(second)
//         .unwrap()
//         .with_nanosecond(0)
//         .unwrap();
//     Ok(dt)
// }

// #[cfg(not(target_arch = "wasm32"))]
// pub fn fmt_picker_datetime_to_dt(picker_datetime: &str) -> Result<DateTime<Utc>, error::ApiError> {
//     use error::{ApiError, ApiErrorType};
//
//     let err = || ApiError::new(ApiErrorType::BadRequest, "cannot parse picker date string");
//     let (date, time) = picker_datetime.split_once('T').ok_or(err())?;
//     fmt_picker_to_dt(date, time)
// }
//
// pub fn fmt_picker_date_to_dt(date_str: &str) -> DateTime<Utc> {
//     let (year, rest) = date_str.split_once('-').unwrap();
//     let (month, day) = rest.split_once('-').unwrap();
//
//     let year = year.parse::<i32>().unwrap();
//     let month = month.parse::<u32>().unwrap();
//     let day = day.parse::<u32>().unwrap();
//
//     Utc::now()
//         .with_year(year)
//         .unwrap()
//         .with_month(month)
//         .unwrap()
//         .with_day(day)
//         .unwrap()
//         .with_hour(0)
//         .unwrap()
//         .with_minute(0)
//         .unwrap()
//         .with_second(0)
//         .unwrap()
//         .with_nanosecond(0)
//         .unwrap()
// }

pub fn get_cookie(key: &str) -> Option<String> {
    let cookie_str = get_document().cookie().unwrap_or_default();
    wasm_cookies::cookies::get(&cookie_str, key).and_then(|r| r.ok())
}

pub fn set_cookie(key: &str, value: &str, exp: chrono::DateTime<chrono::Utc>) {
    let opts = wasm_cookies::cookies::CookieOptions {
        path: Some("/"),
        domain: None,
        expires: Some(exp.to_rfc2822().into()),
        secure: false,
        // secure: true,
        same_site: SameSite::Lax,
    };
    let cookie_str = wasm_cookies::cookies::set(key, value, &opts);
    get_document()
        .set_cookie(&cookie_str)
        .expect("setting cookie");
}

pub fn get_document() -> HtmlDocument {
    document().unchecked_into::<HtmlDocument>()
}

// #[allow(dead_code)]
// pub fn get_path() -> String {
//     location().pathname().unwrap_or_default()
// }

pub fn copy_to_clip(text: &str) {
    match window().navigator().clipboard() {
        None => {
            error!("No access to the clipboard");
        }
        Some(clipboard) => {
            let _ = clipboard.write_text(text);
        }
    }
}

// pub fn save_to_local_storage(key: &str, value: &str) {
//     if let Ok(Some(storage)) = window().local_storage() {
//         if let Err(err) = storage.set_item(key, value) {
//             error!("error saving value in localStorage: {:?}", err);
//         }
//     }
// }
//
// pub fn get_from_local_storage(key: &str) -> Option<String> {
//     if let Ok(Some(storage)) = window().local_storage() {
//         storage.get_item(key).unwrap_or(None)
//     } else {
//         None
//     }
// }
//
// pub fn delete_from_local_storage(key: &str) {
//     if let Ok(Some(storage)) = window().local_storage() {
//         let _ = storage.remove_item(key);
//     }
// }

// pub fn today_midnight() -> DateTime<Utc> {
//     Utc::now()
//         .with_hour(0)
//         .unwrap()
//         .with_minute(0)
//         .unwrap()
//         .with_second(0)
//         .unwrap()
//         .with_nanosecond(0)
//         .unwrap()
// }
