use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use std::time::Duration;

use super::*;
use crate::controller::Event;
use crate::input::jobs::Job;

#[derive(Deserialize, Debug)]
pub struct AlarmDataMinFrom {
    min_till_alarm: String,
}

pub fn set_alarm_minutes_from_now(
    req: HttpRequest,
    params: Form<AlarmDataMinFrom>,
    state: Data<State>,
    auth: BasicAuth,
) -> HttpResponse {
    if authenticated(auth) {
        //Code to parse alarm time
        dbg!(&params);

        if let Ok(minutes) = params.min_till_alarm.parse() {
            let time = Utc::now() + chrono::Duration::minutes(minutes);

            //state.alarms.add_alarm(time).unwrap();
            HttpResponse::Ok().finish()
        } else {
            make_error(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        make_auth_error()
    }
}

#[derive(Deserialize, Debug)]
pub struct AlarmDataUnixTS {
    timestamp: String,
}

pub async fn set_alarm_unix_timestamp(
    params: Form<AlarmDataUnixTS>,
    state: Data<State>,
) -> HttpResponse {
    //Code to parse alarm time
    dbg!(&params);

    if let Ok(ts) = params.timestamp.parse() {
        let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ts, 0), Utc);
        dbg!(time);
        dbg!(Utc::now());

        if time > Utc::now() {
            let alarm = Job::from(time, Event::Alarm, Some(Duration::from_secs(3600 * 2)));

            if state.jobs.add_alarm(alarm).await.is_ok() {
                dbg!("done setting alarm");
                HttpResponse::Ok().finish()
            } else {
                dbg!();
                make_error(StatusCode::INTERNAL_SERVER_ERROR)
            }
        } else {
            dbg!();
            make_error(StatusCode::UNPROCESSABLE_ENTITY)
        }
    } else {
        dbg!();
        make_error(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub fn list_alarms(state: Data<State>) -> HttpResponse {
    //Code to parse alarm time

    let alarms = state.jobs.list();
    let mut list = String::with_capacity(alarms.len() * 100);
    for (id, alarm) in alarms {
        list.push_str(&format!(
            "{:x}, {}, {:?}, {:?}",
            id,
            &alarm.time.to_rfc2822(),
            &alarm.action,
            &alarm.expiration,
        ));
        list.push_str("\n");
    }
    dbg!(&list);
    HttpResponse::Ok().body(list)
}