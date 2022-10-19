mod logger;

use jni::objects::JObject;
use jni::objects::JString;
use jni::JNIEnv;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Sender;
use unftp_sbe_fs::ServerExt;

static INSTANCE: Mutex<Option<Sender<()>>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_00024Companion_init(
    _env: JNIEnv,
    _thiz: JObject,
) {
    logger::init_logger();
}

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_up(
    env: JNIEnv,
    _thiz: JObject,
    path: JString,
) {
    let path: String = env
        .get_string(path)
        .expect("Couldn't get java string.")
        .into();

    {
        let ins = INSTANCE.lock().unwrap();
        if let Some(_) = *ins {
            return;
        }
    }

    let rt = Runtime::new().unwrap();
    let (s, mut r) = channel(1);

    {
        let mut ins = INSTANCE.lock().unwrap();
        *ins = Some(s);
    }

    rt.spawn(async move {
        let ftp_home = PathBuf::from(path);
        let server = libunftp::Server::with_fs(ftp_home)
            .greeting("Welcome to my FTP server")
            .passive_ports(50000..65535);
        server.listen("0.0.0.0:2121").await.unwrap();
    });

    r.blocking_recv();

    {
        let mut ins = INSTANCE.lock().unwrap();
        *ins = None;
    }

    rt.shutdown_background();
}

#[no_mangle]
pub extern "C" fn Java_com_atstudio_denort_utils_Ftpd_down(_env: JNIEnv, _thiz: JObject) {
    let ins = INSTANCE.lock().unwrap();
    if let Some(s) = &*ins {
        s.try_send(()).unwrap();
    }
}
