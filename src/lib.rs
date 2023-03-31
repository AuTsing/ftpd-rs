mod ftpd;

use an_logger::init_logger_for_log_write;
use ftpd::start_server;
use jni::objects::JObject;
use jni::objects::JString;
use jni::JNIEnv;
use libunftp::ServerError;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Sender;

static INSTANCE: Mutex<Option<Sender<Result<(), ServerError>>>> = Mutex::new(None);

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
    _thiz: JObject,
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

    let result = rt.block_on(async {
        tokio::spawn(future);
        rx.recv().await.unwrap()
    });

    println!("result: {:?}", result);

    if let Err(e) = result {
        env.throw_new("java/lang/Exception", e.to_string()).unwrap();
    }

    {
        let mut ins = INSTANCE.lock().unwrap();
        *ins = None;
    }
}

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_stop(_env: JNIEnv, _thiz: JObject) {
    let ins = INSTANCE.lock().unwrap();
    if let Some(tx) = &*ins {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { tx.send(Ok(())).await.unwrap() });
    }
}
