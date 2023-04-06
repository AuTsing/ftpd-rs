mod ftpd;

use an_logger::init_logger_for_log_write;
use ftpd::start_server;
use jni::objects::JObject;
use jni::objects::JString;
use jni::objects::JValue;
use jni::JNIEnv;
use libunftp::ServerError;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Sender;

static INSTANCE: Mutex<Option<Sender<Result<(), ServerError>>>> = Mutex::new(None);

fn set_exit_code(env: &mut JNIEnv, thiz: &JObject, exit_code: i32) {
    env.set_field(thiz, "exitCode", "I", JValue::from(exit_code))
        .unwrap();
}

fn set_exception(env: &mut JNIEnv, thiz: &JObject, message: String) {
    let exception = env
        .new_object(
            "java/lang/Exception",
            "(Ljava/lang/String;)V",
            &[JValue::from(&env.new_string(message).unwrap())],
        )
        .unwrap();
    env.set_field(
        thiz,
        "exception",
        "Ljava/lang/Exception;",
        JValue::from(&exception),
    )
    .unwrap();
}

fn set_running(env: &mut JNIEnv, thiz: &JObject, running: bool) {
    env.call_method(thiz, "updateRunning", "(Z)V", &[JValue::from(running)])
        .unwrap();
}

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_00024Companion_init(
    _env: JNIEnv,
    _thiz: JObject,
) {
    #[cfg(debug_assertions)]
    init_logger_for_log_write(b"FtpdLib\0");
}

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_run(
    mut env: JNIEnv,
    thiz: JObject,
    host: JString,
    port: i32,
    path: JString,
) {
    {
        let ins = INSTANCE.lock().unwrap();
        if let Some(_) = *ins {
            return;
        }
    }

    let host = env.get_string(&host).unwrap().to_string_lossy().to_string();
    let path = env.get_string(&path).unwrap().to_string_lossy().to_string();

    let rt = Runtime::new().unwrap();
    let (tx, mut rx) = channel::<Result<(), ServerError>>(2);

    {
        let mut ins = INSTANCE.lock().unwrap();
        *ins = Some(tx.clone());
    }

    let future = async move {
        let run_result = start_server(host, port, path).await;
        tx.send(run_result).await.unwrap();
    };

    set_running(&mut env, &thiz, true);

    let result = rt.block_on(async {
        tokio::spawn(future);
        rx.recv().await.unwrap()
    });

    match result {
        Ok(_) => {
            set_exit_code(&mut env, &thiz, 0);
            set_running(&mut env, &thiz, false);
        }
        Err(e) => {
            set_exit_code(&mut env, &thiz, 1);
            set_exception(&mut env, &thiz, format!("{:?}", e));
            set_running(&mut env, &thiz, false);
        }
    }

    {
        let mut ins = INSTANCE.lock().unwrap();
        *ins = None;
    }
}

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_stop(mut _env: JNIEnv, _thiz: JObject) {
    let ins = INSTANCE.lock().unwrap();
    if let Some(tx) = &*ins {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { tx.send(Ok(())).await.unwrap() });
    }
}
