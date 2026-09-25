// 入口为 ui/main.slint；ui/tray.slint 由 main.slint 导入，会在同一次编译中一并生成。
fn main() {
    slint_build::compile("ui/main.slint").expect("failed to compile ui/main.slint");
}
