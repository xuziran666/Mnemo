// 防止在 Windows 发布版中出现额外的控制台窗口，切勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mnemo_lib::run()
}
