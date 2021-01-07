#[macro_use]
extern crate log;

use std::sync::{mpsc, Once};
use std::{any, panic, thread};

use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::sys::{jfloat, jint, jlong};
use jni::{JNIEnv, JavaVM};

use bwt::error::{Context, Error, Result};
use bwt::util::bitcoincore_ext::Progress;
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

        let (progress_tx, progress_rx) = mpsc::channel();
        spawn_recv_progress_thread(progress_rx, jvm, callback_g);

        env.call_method(callback, "onBooting", "()V", &[]).unwrap();
        let app = App::boot(config, Some(progress_tx))?;

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

        let (shutdown_tx, shutdown_rx) = mpsc::sync_channel(1);
        let shutdown_handler = ShutdownHandler(shutdown_tx);
        let shutdown_ptr = Box::into_raw(Box::new(shutdown_handler)) as jlong;

        env.call_method(callback, "onReady", "(J)V", &[shutdown_ptr.into()])
            .unwrap();

        app.sync(Some(shutdown_rx));

        Ok(())
    };

    if let Err(e) = panic::catch_unwind(start)
        .map_err(fmt_panic)
        .and_then(|r| r.map_err(fmt_error))
    {
        warn!("{}", e);
        env.throw_new("dev/bwt/libbwt/BwtException", &e).unwrap();
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

    if let Err(e) = panic::catch_unwind(test)
        .map_err(fmt_panic)
        .and_then(|r| r.map_err(fmt_error))
    {
        warn!("test rpc failed: {:?}", e);
        env.throw_new("dev/bwt/libbwt/BwtException", &e.to_string())
            .unwrap();
    }
}

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
                Err(mpsc::RecvError) => break,
            }
        }
    });
    // wait for the thread to start
    rx.recv().unwrap();

    handle
}

fn fmt_error(e: Error) -> String {
    let causes: Vec<String> = e.chain().map(|cause| cause.to_string()).collect();
    causes.join(": ")
}

fn fmt_panic(err: Box<dyn any::Any + Send + 'static>) -> String {
    format!(
        "panic: {}",
        if let Some(s) = err.downcast_ref::<&str>() {
            s
        } else if let Some(s) = err.downcast_ref::<String>() {
            s
        } else {
            "unknown panic"
        }
    )
}
