use sysinfo::SystemExt;

fn main() {
    let s = sysinfo::System::new();
    let process = s.get_process(953238);
    println!("{:?}", process);
}
