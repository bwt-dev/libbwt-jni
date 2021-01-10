#[macro_use]
extern crate log;

use std::sync::{mpsc, Once};
use std::{any, panic, thread};

use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::sys::{jfloat, jint, jlong};
use jni::{JNIEnv, JavaVM};

use bwt::error::{BwtError, Context, Error, Result};
use bwt::util::{bitcoincore_wait::Progress, on_oneshot_done};
use bwt::{App, Config};

static INIT_LOGGER: Once = Once::new();

#[repr(C)]
pub struct ShutdownHandler(mpsc::SyncSender<()>);

#[no_mangle]
pub extern "system" fn Java_dev_bwt_libbwt_daemon_NativeBwtDaemon_start(
    env: JNIEnv,
    _: JClass,
    json_config: JString,
    callback: JObject,
) {
    let json_config: String = env.get_string(json_config).unwrap().into();

    let jvm = env.get_java_vm().unwrap();
    let callback_g = env.new_global_ref(callback).unwrap();

    let start = || -> Result<_> {
        let config: Config = serde_json::from_str(&json_config).context("Invalid config")?;
        // The verbosity level cannot be changed once enabled.
        INIT_LOGGER.call_once(|| config.setup_logger());

        // Spawn background thread to emit syncing/scanning progress updates
        let (progress_tx, progress_rx) = mpsc::channel();
        spawn_recv_progress_thread(progress_rx, jvm, callback_g);

        // Setup shutdown channel and pass the shutdown handler to onBooting
        let (shutdown_tx, shutdown_rx) = make_shutdown_channel(progress_tx.clone());
        let shutdown_ptr = ShutdownHandler(shutdown_tx).into_raw() as jlong;
        env.call_method(callback, "onBooting", "(J)V", &[shutdown_ptr.into()])
            .unwrap();

        // Start up bwt, run the initial sync and start the servers
        let app = App::boot(config, Some(progress_tx))?;

        if shutdown_rx.try_recv() != Err(mpsc::TryRecvError::Empty) {
            return Err(BwtError::Canceled.into());
        }

        #[cfg(feature = "electrum")]
        if let Some(addr) = app.electrum_addr() {
            let addr = env.new_string(addr.to_string()).unwrap().into_inner();
            env.call_method(
                callback,
                "onElectrumReady",
                "(Ljava/lang/String;)V",
                &[addr.into()],
            )
            .unwrap();
        }
        #[cfg(feature = "http")]
        if let Some(addr) = app.http_addr() {
            let addr = env.new_string(addr.to_string()).unwrap().into_inner();
            env.call_method(
                callback,
                "onHttpReady",
                "(Ljava/lang/String;)V",
                &[addr.into()],
            )
            .unwrap();
        }

        env.call_method(callback, "onReady", "()V", &[]).unwrap();

        app.sync(Some(shutdown_rx));

        Ok(())
    };

    if let Err(e) = try_run(start) {
        warn!("{}", e);
        env.throw_new("dev/bwt/libbwt/BwtException", &e).unwrap();
    } else {
        debug!("daemon stopped successfully");
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_dev_bwt_libbwt_daemon_NativeBwtDaemon_shutdown(
    _env: JNIEnv,
    _: JClass,
    shutdown_ptr: jlong,
) {
    // Take ownership and drop it. This will disconnect the mpsc channel and shutdown the app.
    Box::from_raw(shutdown_ptr as *mut ShutdownHandler);
}

#[no_mangle]
pub extern "system" fn Java_dev_bwt_libbwt_daemon_NativeBwtDaemon_testRpc(
    env: JNIEnv,
    _: JClass,
    json_config: JString,
) {
    let json_config: String = env.get_string(json_config).unwrap().into();

    let test = || App::test_rpc(&serde_json::from_str(&json_config)?);

    if let Err(e) = try_run(test) {
        warn!("test rpc failed: {:?}", e);
        env.throw_new("dev/bwt/libbwt/BwtException", &e.to_string())
            .unwrap();
    }
}

impl ShutdownHandler {
    fn into_raw(self) -> *const ShutdownHandler {
        Box::into_raw(Box::new(self))
    }
}

// Run the closure, handling panics and errors, and formatting them to a string
fn try_run<F>(f: F) -> std::result::Result<(), String>
where
    F: FnOnce() -> Result<()> + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Err(panic) => Err(fmt_panic(panic)),
        // Consider user-initiated cancellations (via `bwt_shutdown`) as successful termination
        Ok(Err(e)) if matches!(e.downcast_ref::<BwtError>(), Some(BwtError::Canceled)) => Ok(()),
        Ok(Err(e)) => Err(fmt_error(e)),
        Ok(Ok(())) => Ok(()),
    }
}

// Spawn a thread to receive mpsc progress updates and forward them to on{Sync,Scan}Progress
fn spawn_recv_progress_thread(
    progress_rx: mpsc::Receiver<Progress>,
    jvm: JavaVM,
    callback: GlobalRef,
) -> thread::JoinHandle<()> {
    let (tx, rx) = mpsc::sync_channel(1);
    let handle = thread::spawn(move || {
        tx.send(()).unwrap();
        let env = jvm.attach_current_thread().unwrap();
        loop {
            match progress_rx.recv() {
                Ok(Progress::Sync { progress_n, tip }) => {
                    let progress_n = progress_n as jfloat;
                    let tip = tip as jint;
                    env.call_method(
                        &callback,
                        "onSyncProgress",
                        "(FI)V",
                        &[progress_n.into(), tip.into()],
                    )
                    .unwrap();
                }
                Ok(Progress::Scan { progress_n, eta }) => {
                    let progress_n = progress_n as jfloat;
                    let eta = eta as jint;
                    env.call_method(
                        &callback,
                        "onScanProgress",
                        "(FI)V",
                        &[progress_n.into(), eta.into()],
                    )
                    .unwrap();
                }
                Err(mpsc::RecvError) | Ok(Progress::Done) => break,
            }
        }
    });
    // wait for the thread to start
    rx.recv().unwrap();

    handle
}

fn make_shutdown_channel(
    progress_tx: mpsc::Sender<Progress>,
) -> (mpsc::SyncSender<()>, mpsc::Receiver<()>) {
    let (shutdown_tx, shutdown_rx) = mpsc::sync_channel(1);

    // When the shutdown signal is received, we need to emit a Progress::Done
    // message to stop the progress recv thread, which will disconnect the
    // progress channel and stop the bwt start-up procedure.
    let shutdown_rx = on_oneshot_done(shutdown_rx, move || {
        progress_tx.send(Progress::Done).ok();
    });

    (shutdown_tx, shutdown_rx)
}

fn fmt_error(err: Error) -> String {
    let causes: Vec<String> = err.chain().map(|cause| cause.to_string()).collect();
    causes.join(": ")
}

fn fmt_panic(err: Box<dyn any::Any + Send + 'static>) -> String {
    if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.to_string()
    } else {
        "unknown panic".to_string()
    }
}
