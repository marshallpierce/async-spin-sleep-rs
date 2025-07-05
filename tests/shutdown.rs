use async_spin_sleep::Report;
use futures::future::join_all;
use std::time;

#[tokio::test]
async fn shutdown_with_no_timers_works() {
    let (handle, driver) = async_spin_sleep::create();
    let driver_handle = std::thread::spawn(driver);

    let start = time::Instant::now();

    handle.shutdown();

    driver_handle.join().unwrap();

    let elapsed = start.elapsed();
    assert!(elapsed < time::Duration::from_millis(100), "elapsed: {elapsed:?}");
}

#[tokio::test]
async fn shutdown_fires_existing_later_timers() {
    let (handle, driver) = async_spin_sleep::create();
    let driver_handle = std::thread::spawn(driver);

    // schedule some tasks waiting for timers in the future, after the driver is shut down
    let handles = (1..=100)
        .map(|s| tokio::spawn(handle.sleep_for(time::Duration::from_secs(s))))
        .collect::<Vec<_>>();

    let before_shutdown = time::Instant::now();

    handle.shutdown();

    driver_handle.join().unwrap();

    let driver_join_elapsed = before_shutdown.elapsed();
    assert!(
        driver_join_elapsed < time::Duration::from_millis(100),
        "elapsed: {driver_join_elapsed:?}"
    );

    // all existing timers should fire quickly
    let reports =
        join_all(handles.into_iter()).await.into_iter().map(|res| res.unwrap()).collect::<Vec<_>>();
    let timers_complete_elapsed = before_shutdown.elapsed();
    assert!(
        timers_complete_elapsed < time::Duration::from_millis(100),
        "elapsed: {timers_complete_elapsed:?}"
    );

    for rep in reports {
        match rep {
            Report::CompletedEarly(dur) => {
                // the soonest timer was for 1s, so all should have fired early
                assert!(dur > time::Duration::from_millis(200))
            }
            Report::Completed(_) | Report::ExpiredTimer(_) => {
                panic!("Unexpected report: {rep:?}")
            }
        }
    }
}
