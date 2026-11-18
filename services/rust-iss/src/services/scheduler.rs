use crate::services::{collection, AppState};
pub fn start(st: AppState) {
    if !st.config.scheduler {
        return;
    }
    for &(source, seconds) in &st.config.intervals {
        let st = st.clone();
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(std::time::Duration::from_secs(seconds));
            timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                timer.tick().await;
                let _ = collection::refresh(&st, source).await;
            }
        });
    }
}
