fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let mut success = true;
    tracing::error!("make sure errors show up in CI");
    tracing::info!("Logical CPU count {}", num_cpus::get());
    tracing::info!("Physical CPU count {}", num_cpus::get_physical());

    tracing::info!("Single thread test");
    if ! hammer_thread(0) {
        success = false;
    }

    tracing::info!("Multi-thread test");
    let handles = (0..8).map(|i| std::thread::spawn(move || {
        hammer_thread(i)
    })).collect::<Vec<_>>();


    for handle in handles {
        if ! handle.join().unwrap() {
            success = false;
        }
    }

    if ! success {
        anyhow::bail!("Something failed");
    }
    Ok(())
}

fn hammer_thread(i: u32) -> bool {
    let name = format!("dev.firezone.client/test_OZQP3QIN/token/{i}");
    let mut success = true;

    let Ok(entry) = keyring::Entry::new_with_target(&name, "", "") else {
        tracing::error!("couldn't create keyring::Entry");
        return false;
    };

    for iteration in 0..1000 {
        match hammer_cycle(&name, &entry) {
            Ok(false) => {
                tracing::error!(?iteration, "Multi-thread test failed");
                success = false;
            }
            Ok(true) => {}
            Err(error) => {
                tracing::error!(?error);
                success = false;
            }
        }
    }

    keyring::Entry::new_with_target(&name, "", "").unwrap().delete_password().ok();
    success
}

fn hammer_cycle(name: &str, entry: &keyring::Entry) -> anyhow::Result<bool> {
    let mut success = true;
    let password = "bogus_password";

    entry.delete_password().ok();
    entry.set_password(password)?;

    for iteration in 0..5 {
        let Ok(actual) = entry.get_password()
        else {
            tracing::error!(?name, ?iteration, "couldn't get_password");
            success = false;
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        };
        if actual != password {
           anyhow::bail!("retrieved password didn't match stored password");
        }
        break;
    }

    Ok(success)
}
