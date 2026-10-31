// 入口为 ui/main.slint；ui/tray.slint 自 0.4.1 起不再参与编译（托盘已移除）。
fn main() {
    slint_build::compile("ui/main.slint").expect("failed to compile ui/main.slint");
}
