pub mod cmd {
    pub fn run_cmd(input_cmd: &str) {
        println!("---------------------------------------------");
        let output = std::process::Command::new("bash")
            .arg("-c").arg(&format!("{}", input_cmd))
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Command succeeded:\n{}", stdout);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Command failed:\n{}", stderr);
        }
    }
}
