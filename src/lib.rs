mod ftpd;

use an_logger::init_logger_for_log_write;
use ftpd::start_server;
use jni::objects::JObject;
use jni::objects::JString;
use jni::objects::JValue;
use jni::JNIEnv;
use libunftp::ServerError;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio::task::LocalSet;

static INSTANCE: Mutex<Option<Arc<Mutex<JoinHandle<Result<(), ServerError>>>>>> = Mutex::new(None);

fn set_running(env: &JNIEnv, thiz: &JObject, status: bool) {
    env.call_method(*thiz, "changeRunning", "(Z)V", &[JValue::from(status)])
        .expect("Couldn't call method changeRunning");
}

fn set_exit_code(env: &JNIEnv, thiz: &JObject, exit_code: i32) {
    env.set_field(*thiz, "exitCode", "I", JValue::from(exit_code))
        .expect("Couldn't set field exitCode");
}

fn set_exception(env: &JNIEnv, thiz: &JObject, message: &String) {
    let message_value: JValue = env
        .new_string(message)
        .expect("Couldn't new string of message_value")
        .into();
    let exception: JValue = env
        .new_object(
            "java/lang/Exception",
            "(Ljava/lang/String;)V",
            &[message_value],
        )
        .expect("Couldn't new object of exception")
        .into();
    env.set_field(*thiz, "exception", "Ljava/lang/Exception;", exception)
        .expect("Couldn't set field exception");
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
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_up(
    env: JNIEnv,
    thiz: JObject,
    path: JString,
) {
    {
        let ins = INSTANCE.lock().unwrap();
        if let Some(_) = *ins {
            return;
        }
    }

    let path: String = env
        .get_string(path)
        .expect("Couldn't get string of path")
        .into();

    let rt = Runtime::new().unwrap();
    let local = LocalSet::new();

    let handle = Arc::new(Mutex::new(local.spawn_local(start_server(path))));

    {
        let mut ins = INSTANCE.lock().unwrap();
        *ins = Some(handle.clone());
    }

    set_running(&env, &thiz, true);

    rt.block_on(async {
        local.await;

        let mut handle = handle.lock().unwrap();
        let handle_result = (&mut *handle).await;
        if let Err(e) = handle_result {
            let message = e.to_string();
            set_exit_code(&env, &thiz, 2);
            set_exception(&env, &thiz, &message);
            return;
        }

        let run_result = handle_result.unwrap();
        if let Err(e) = run_result {
            let message = e.to_string();
            set_exit_code(&env, &thiz, 1);
            set_exception(&env, &thiz, &message);
            return;
        }

        set_exit_code(&env, &thiz, 0);
    });

    {
        let mut ins = INSTANCE.lock().unwrap();
        *ins = None;
    }

    set_running(&env, &thiz, false);
}

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_down(_env: JNIEnv, _thiz: JObject) {
    let ins = INSTANCE.lock().unwrap();
    if let Some(handle) = &*ins {
        handle.lock().unwrap().abort();
    }
}
